use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct JiraTicketInfo {
    pub key: String,
    pub status: String,
    pub ready: bool,
}

#[async_trait]
pub trait JiraService: Sync + Send {
    async fn get_jira_issues(&self, issue_keys: Vec<String>)
        -> Result<Vec<JiraTicketInfo>, String>;
}

pub struct JiraServiceImpl {
    pub token: String,
    pub base_url: String,
}

impl JiraServiceImpl {
    pub fn new(base_url: String, token: String) -> Self {
        JiraServiceImpl { base_url, token }
    }
}

#[async_trait]
impl JiraService for JiraServiceImpl {
    async fn get_jira_issues(
        &self,
        issue_keys: Vec<String>,
    ) -> Result<Vec<JiraTicketInfo>, String> {
        let base_url = &self.base_url;
        let token = &self.token;

        if issue_keys.is_empty() {
            return Err("No issue keys provided".to_string());
        }

        let issues_str = issue_keys.join(",");

        let response = reqwest::Client::new()
            .get(&format!("{}/rest/api/2/search", base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .query(&[("jql", &format!("key in ({})", issues_str))])
            .query(&[("maxResults", "100")])
            .send()
            .await
            .map_err(|e| e.to_string());

        if let Err(err) = response {
            eprintln!("Error fetching JIRA issues: {:?}", err);
            return Err(err);
        }

        let response = response.unwrap();
        if !response.status().is_success() {
            eprintln!(
                "Error fetching JIRA issues: {:?}",
                response.text().await.unwrap()
            );
            return Err("Error fetching JIRA issues".to_string());
        }

        let json_result = response.json().await.map_err(|e| e.to_string());
        if let Err(err) = json_result {
            eprintln!("Error fetching JIRA issues: {:?}", err);
            return Err(err);
        }

        let json: serde_json::Value = json_result.unwrap();
        if !json["issues"].is_array() {
            return Err("Error fetching JIRA issues".to_string());
        }

        if json["issues"].as_array().unwrap().is_empty() {
            return Ok(vec![]);
        }

        json["issues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|issue| {
                let key = issue["key"].as_str().unwrap().to_string();
                let status = if issue["fields"]["status"]["name"].is_string() {
                    issue["fields"]["status"]["name"]
                        .as_str()
                        .unwrap()
                        .to_string()
                } else {
                    "".to_string()
                };

                let ready = if issue["fields"]["customfield_19899"]["value"].is_string() {
                    issue["fields"]["customfield_19899"]["value"]
                        .as_str()
                        .unwrap()
                        == "Go"
                } else {
                    false
                };

                Ok(JiraTicketInfo { key, status, ready })
            })
            .collect()
    }
}

mod tests {
    use crate::api::jira_service::{JiraService, JiraServiceImpl};

    #[tokio::test]
    async fn unit_test_get_jira_issue() {
        let jira_service = JiraServiceImpl::new(
            "https://jira.my-company.net".to_string(),
            "token".to_string(),
        );
        let payload = vec!["BAR-1771".to_string(), "BAR-1583".to_string()];

        let result = jira_service.get_jira_issues(payload).await;

        assert!(result.is_ok());

        let mut issues = result.unwrap();
        issues.sort_by_key(|x| x.key.clone()); // so that the order is consistent

        assert_eq!(issues.len(), 2);

        assert_eq!(issues[0].key, "BAR-1583");
        assert_eq!(issues[0].status, "Open");
        assert_eq!(issues[0].ready, false);

        assert_eq!(issues[1].key, "BAR-1771");
        assert_eq!(issues[1].status, "In Progress");
        assert_eq!(issues[1].ready, false);
    }
}
