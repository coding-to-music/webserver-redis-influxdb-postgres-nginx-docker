use std::fmt::Display;
use webserver_contracts::{JsonRpcRequest, JsonRpcResponse};

pub struct WebserverClient {
    url: String,
    client: reqwest::Client,
}

impl WebserverClient {
    pub fn new() -> WebserverClientBuilder {
        WebserverClientBuilder::default()
    }

    fn from_builder(builder: WebserverClientBuilder) -> Result<Self, WebserverBuilderError> {
        let url = builder.url.ok_or(WebserverBuilderError::MissingUrl)?;
        let client = reqwest::Client::new();

        Ok(Self { url, client })
    }

    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebserverClientError> {
        let response: JsonRpcResponse = self
            .client
            .post(&self.url)
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn send_batch(
        &self,
        requests: Vec<JsonRpcRequest>,
    ) -> Result<Vec<JsonRpcResponse>, WebserverClientError> {
        let response: Vec<JsonRpcResponse> = self
            .client
            .post(&self.url)
            .body(serde_json::to_string(&requests).unwrap())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}

pub struct WebserverClientBuilder {
    url: Option<String>,
}

impl WebserverClientBuilder {
    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn build(self) -> Result<WebserverClient, WebserverBuilderError> {
        WebserverClient::from_builder(self)
    }
}

impl Default for WebserverClientBuilder {
    fn default() -> Self {
        Self { url: None }
    }
}

#[derive(Debug)]
pub enum WebserverBuilderError {
    MissingUrl,
}

impl Display for WebserverBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            WebserverBuilderError::MissingUrl => "missing url",
        };

        write!(f, "{}", output)
    }
}

#[derive(Debug)]
pub enum WebserverClientError {
    ReqwestError(reqwest::Error),
}

impl Display for WebserverClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            WebserverClientError::ReqwestError(e) => format!("reqwest error: '{}'", e),
        };

        write!(f, "{}", output)
    }
}

impl From<reqwest::Error> for WebserverClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}
