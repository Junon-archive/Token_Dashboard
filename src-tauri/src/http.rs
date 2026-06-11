use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};
use thiserror::Error;

use crate::config::{validate_endpoint_url, EndpointError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Error)]
pub enum UsageHttpError {
    #[error("endpoint rejected: {0}")]
    Endpoint(#[from] EndpointError),
    #[error("invalid HTTP header")]
    InvalidHeader,
    #[error("network request failed")]
    Network,
}

#[async_trait]
pub trait UsageHttpClient: Send + Sync {
    async fn get_with_bearer(
        &self,
        url: &str,
        bearer_token: &str,
        headers: &[SafeHeader],
    ) -> Result<UsageResponse, UsageHttpError>;
}

#[derive(Debug, Default, Clone)]
pub struct ReqwestUsageHttpClient {
    client: reqwest::Client,
}

#[async_trait]
impl UsageHttpClient for ReqwestUsageHttpClient {
    async fn get_with_bearer(
        &self,
        url: &str,
        bearer_token: &str,
        headers: &[SafeHeader],
    ) -> Result<UsageResponse, UsageHttpError> {
        validate_endpoint_url(url)?;

        let mut header_map = HeaderMap::new();
        header_map.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {bearer_token}"))
                .map_err(|_| UsageHttpError::InvalidHeader)?,
        );

        for header in headers {
            let name = HeaderName::from_bytes(header.name.as_bytes())
                .map_err(|_| UsageHttpError::InvalidHeader)?;
            let value =
                HeaderValue::from_str(&header.value).map_err(|_| UsageHttpError::InvalidHeader)?;
            header_map.insert(name, value);
        }

        if !header_map.contains_key(USER_AGENT) {
            header_map.insert(
                USER_AGENT,
                HeaderValue::from_static("token-dashboard/0.1.0"),
            );
        }

        let response = self
            .client
            .get(url)
            .headers(header_map)
            .send()
            .await
            .map_err(|_| UsageHttpError::Network)?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|_| UsageHttpError::Network)?;

        Ok(UsageResponse { status, body })
    }
}

#[cfg(test)]
pub mod testsupport {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    pub struct RecordedRequest {
        pub url: String,
        pub bearer_was_attached: bool,
    }

    #[derive(Debug, Clone)]
    pub struct FixtureHttpClient {
        response: UsageResponse,
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    impl FixtureHttpClient {
        pub fn new(status: u16, body: impl Into<String>) -> Self {
            Self {
                response: UsageResponse {
                    status,
                    body: body.into(),
                },
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn requests(&self) -> Vec<RecordedRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl UsageHttpClient for FixtureHttpClient {
        async fn get_with_bearer(
            &self,
            url: &str,
            bearer_token: &str,
            _headers: &[SafeHeader],
        ) -> Result<UsageResponse, UsageHttpError> {
            validate_endpoint_url(url)?;
            self.requests.lock().unwrap().push(RecordedRequest {
                url: url.to_string(),
                bearer_was_attached: !bearer_token.is_empty(),
            });
            Ok(self.response.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EndpointError;
    use testsupport::FixtureHttpClient;

    #[tokio::test]
    async fn fixture_client_rejects_endpoint_before_recording_token_request() {
        let client = FixtureHttpClient::new(200, "{}");
        let result = client
            .get_with_bearer("https://evil.invalid/usage", "synthetic-access", &[])
            .await;

        assert!(matches!(
            result,
            Err(UsageHttpError::Endpoint(EndpointError::HostNotAllowed))
        ));
        assert!(client.requests().is_empty());
    }
}
