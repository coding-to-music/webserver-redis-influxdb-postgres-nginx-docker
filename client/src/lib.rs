use async_mutex::Mutex;
use chrono::{DateTime, Utc};
use contracts::{GetTokenRequest, GetTokenResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use isahc::{http::method, AsyncReadResponseExt};
use std::{error::Error, fmt::Display, sync::Arc};

#[macro_use]
extern crate log;

pub struct WebserverClient {
    url: String,
    client: isahc::HttpClient,
    token_manager: Arc<Mutex<TokenManager>>,
}

impl WebserverClient {
    pub fn new(url: String, key_name: String, key_value: String) -> WebserverClientBuilder {
        WebserverClientBuilder::new(url, key_name, key_value)
    }

    fn from_builder(builder: WebserverClientBuilder) -> Result<Self, WebserverBuilderError> {
        let mut url = builder.url.trim();
        if let Some(without_trailing_slash) = url.strip_suffix("/") {
            url = without_trailing_slash;
        }

        if url.is_empty() {
            return Err(WebserverBuilderError::InvalidUrl);
        }

        let client = isahc::HttpClient::new().unwrap();

        let key_name = builder.key_name.trim();
        if key_name.is_empty() {
            return Err(WebserverBuilderError::InvalidKeyName);
        }

        let key_value = builder.key_value.trim();
        if key_value.is_empty() {
            return Err(WebserverBuilderError::InvalidKeyValue);
        }

        let token_manager = Arc::new(Mutex::new(TokenManager::new(
            key_name.to_owned(),
            key_value.to_owned(),
        )));

        Ok(Self {
            url: url.to_owned(),
            client,
            token_manager,
        })
    }

    fn api_url(&self) -> String {
        format!("{}/api", self.url)
    }

    fn token_url(&self) -> String {
        format!("{}/api/token", self.url)
    }

    pub async fn token(&self) -> Result<String, WebserverClientError> {
        let mut lock = self.token_manager.lock().await;
        lock.refresh(&self).await?;
        Ok(lock.token.clone().unwrap())
    }

    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebserverClientError> {
        let requests = vec![request];
        let mut responses = self.send_batch(requests).await?;
        Ok(responses.remove(0))
    }

    pub async fn send_batch(
        &self,
        requests: Vec<JsonRpcRequest>,
    ) -> Result<Vec<JsonRpcResponse>, WebserverClientError> {
        let token;
        {
            token = self.token().await?;
        }

        let http_request = isahc::Request::builder()
            .uri(self.api_url())
            .method(method::Method::POST)
            .header("Authorization", format!("Bearer {}", token))
            .body(serde_json::to_vec(&requests)?)?;

        let response: Vec<JsonRpcResponse> =
            self.client.send_async(http_request).await?.json().await?;

        Ok(response)
    }

    async fn get_token(
        &self,
        request: GetTokenRequest,
    ) -> Result<GetTokenResponse, WebserverClientError> {
        let http_request = isahc::Request::builder()
            .uri(self.token_url())
            .method(method::Method::POST)
            .body(serde_json::to_vec(&request)?)?;

        let response: GetTokenResponse = self.client.send_async(http_request).await?.json().await?;

        Ok(response)
    }
}

pub struct WebserverClientBuilder {
    url: String,
    key_name: String,
    key_value: String,
}

impl WebserverClientBuilder {
    fn new(url: String, key_name: String, key_value: String) -> Self {
        Self {
            url,
            key_name,
            key_value,
        }
    }

    pub fn build(self) -> Result<WebserverClient, WebserverBuilderError> {
        WebserverClient::from_builder(self)
    }
}

#[derive(Debug)]
pub enum WebserverBuilderError {
    InvalidUrl,
    InvalidKeyName,
    InvalidKeyValue,
}

impl Display for WebserverBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            WebserverBuilderError::InvalidUrl => "invalid url",
            WebserverBuilderError::InvalidKeyName => "invalid key name",
            WebserverBuilderError::InvalidKeyValue => "invalid key value",
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug)]
pub enum WebserverClientError {
    IsahcError(isahc::Error),
    HttpError(isahc::http::Error),
    WebserverError(JsonRpcError),
    SerdeError(serde_json::Error),
    TokenError(String),
}

impl Display for WebserverClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            WebserverClientError::IsahcError(e) => format!("isahc error: '{}'", e),
            WebserverClientError::WebserverError(e) => {
                format!("webserver error: '{}'", e.message)
            }
            WebserverClientError::SerdeError(serde_error) => {
                format!("serde error: '{}'", serde_error)
            }
            WebserverClientError::TokenError(message) => {
                format!("token error: '{}'", message)
            }
            WebserverClientError::HttpError(e) => {
                format!("isahc http error: '{}'", e)
            }
        };

        write!(f, "{}", output)
    }
}

impl From<isahc::Error> for WebserverClientError {
    fn from(e: isahc::Error) -> Self {
        Self::IsahcError(e)
    }
}

impl From<isahc::http::Error> for WebserverClientError {
    fn from(e: isahc::http::Error) -> Self {
        Self::HttpError(e)
    }
}

impl From<serde_json::Error> for WebserverClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeError(e)
    }
}

impl From<JsonRpcError> for WebserverClientError {
    fn from(e: JsonRpcError) -> Self {
        Self::WebserverError(e)
    }
}

impl Error for WebserverClientError {}

struct TokenManager {
    key_name: String,
    key_value: String,
    token: Option<String>,
    exp: Option<DateTime<Utc>>,
}

impl TokenManager {
    fn new(key_name: String, key_value: String) -> Self {
        Self {
            key_name,
            key_value,
            token: None,
            exp: None,
        }
    }

    async fn refresh(&mut self, client: &WebserverClient) -> Result<(), WebserverClientError> {
        trace!("refreshing webserver token");
        if self.token.is_none() {
            trace!("no token has been retrieved yet");
            self.force_refresh(client).await
        } else {
            let time = self
                .exp
                .expect("'exp' should not be None if 'token' is Some");
            if chrono::Utc::now() >= time {
                trace!("token has expired");
                self.force_refresh(client).await
            } else {
                trace!("token has not expired");
                Ok(())
            }
        }
    }

    async fn force_refresh(
        &mut self,
        client: &WebserverClient,
    ) -> Result<(), WebserverClientError> {
        info!("forcing token refresh");
        let request = GetTokenRequest::new(self.key_name.clone(), self.key_value.clone());

        let response = client.get_token(request).await?;

        if response.success {
            info!("successfully forced token refresh");
            self.token = Some(response.access_token.unwrap());
            self.exp = Some(
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::seconds(3200))
                    .unwrap(),
            );

            Ok(())
        } else {
            info!("failed to force token refresh");
            let message = response.error_message.unwrap();
            Err(WebserverClientError::TokenError(message))
        }
    }
}
