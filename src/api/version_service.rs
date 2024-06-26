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
            .get(&self.config.versions_live)
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
            jira_token: "def".to_string(),
            jira_server: "http:://jira".to_string(),
            versions_live: "http://live".to_string(),
        };
        let service = VersionService::new(config);
        assert_eq!(service.config.github_server, "http://github");
        assert_eq!(service.config.jira_server, "http:://jira");
        assert_eq!(service.config.versions_live, "http://live");
        assert_eq!(service.config.github_token, "abc");
        assert_eq!(service.config.jira_token, "def");
    }

    #[tokio::test]
    async fn test_get_versions_from_live_url_success() {
        let mut live_server = mockito::Server::new_async().await;
        let mut github_server = mockito::Server::new_async().await;
        let mut jira_server = mockito::Server::new_async().await;

        let _m = live_server
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
            .create_async()
            .await;

        let config = Config {
            github_token: "abc".to_string(),
            github_server: github_server.url(),
            jira_token: "abc".to_string(),
            jira_server: jira_server.url(),
            versions_live: live_server.url(),
        };
        let service = VersionService::new(config);
        let result = service.get_versions_from_live_url().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_get_versions_from_live_url_error() {
        let mut live_server = mockito::Server::new_async().await;
        let mut github_server = mockito::Server::new_async().await;
        let mut jira_server = mockito::Server::new_async().await;

        let _m = live_server
            .mock("GET", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let config = Config {
            github_token: "abc".to_string(),
            github_server: github_server.url(),
            jira_token: "abc".to_string(),
            jira_server: jira_server.url(),
            versions_live: live_server.url(),
        };
        let service = VersionService::new(config);
        let result = service.get_versions_from_live_url().await;

        assert!(result.is_err());
    }
}
