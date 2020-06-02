use webserver_contracts::{JsonRpcRequest, JsonRpcResponse};

pub struct WebserverClient {
    url: String,
    inner_client: reqwest::Client,
}

impl WebserverClient {
    fn new(url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            url,
            inner_client: client,
        }
    }

    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebserverClientError> {
        let response: JsonRpcResponse = self
            .inner_client
            .post(&self.url)
            .body(request.as_formatted_json())
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
        Ok(WebserverClient::new(
            self.url.ok_or(WebserverBuilderError::MissingUrl)?,
        ))
    }
}

pub enum WebserverBuilderError {
    MissingUrl,
}

pub enum WebserverClientError {
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for WebserverClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}
