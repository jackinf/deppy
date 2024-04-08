use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    version: String,
    conf_version: String,
    override_version: Option<String>,
    asset_path: String,
    cdn_hostname: String,
    is_developer: bool,
}

pub struct VersionServiceResponse {
    pub version: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct VersionServiceError {
    code: Option<u16>,
    message: String,
}

pub struct VersionService {
    config: Config,
}

impl VersionService {
    pub fn new(config: Config) -> Self {
        VersionService { config }
    }

    pub async fn get_versions_from_live_url(
        &self,
    ) -> Result<VersionServiceResponse, VersionServiceError> {
        let client = Client::new();
        let response = client
            .get(&self.config.github_server)
            .send()
            .await
            .map_err(|e| VersionServiceError {
                code: Option::from(e.status().unwrap().as_u16()),
                message: e.to_string(),
            })?;

        if !(response.status().is_success()) {
            return Err(VersionServiceError {
                code: None,
                message: response.text().await.unwrap(),
            });
        }

        let api_response: ApiResponse = response.json().await.map_err(|e| VersionServiceError {
            code: None,
            message: e.to_string(),
        })?;

        Ok(VersionServiceResponse {
            version: api_response.version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_new() {
        let config = Config {
            github_token: "abc".to_string(),
            github_server: "http://github".to_string(),
            jira_token: "abc".to_string(),
            jira_server: "http:://jira".to_string(),
            versions_live: "http://live".to_string(),
        };
        let service = VersionService::new(config);
        assert_eq!(service.config.github_server, "http://localhost");
    }

    #[tokio::test]
    async fn test_get_versions_from_live_url_success() {
        let mut server = mockito::Server::new();

        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(
                json!({
                    "version": "1.0.0",
                    "confVersion": "1.0.0",
                    "overrideVersion": null,
                    "assetPath": "/assets",
                    "cdnHostname": "cdn.example.com",
                    "isDeveloper": false
                })
                .to_string(),
            )
            .create();

        let config = Config {
            github_token: "abc".to_string(),
            github_server: "http://github".to_string(),
            jira_token: "abc".to_string(),
            jira_server: "http:://jira".to_string(),
            versions_live: "http://live".to_string(),
        };
        let service = VersionService::new(config);
        let result = service.get_versions_from_live_url().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_get_versions_from_live_url_error() {
        let mut server = mockito::Server::new();

        let _m = server
            .mock("GET", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        let config = Config {
            github_token: "abc".to_string(),
            github_server: "http://github".to_string(),
            jira_token: "abc".to_string(),
            jira_server: "http:://jira".to_string(),
            versions_live: "http://live".to_string(),
        };
        let service = VersionService::new(config);
        let result = service.get_versions_from_live_url().await;

        assert!(result.is_err());
    }
}
