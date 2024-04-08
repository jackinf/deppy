use crate::api::github_service::GithubService;
use anyhow::anyhow;
use async_trait::async_trait;

#[async_trait]
pub trait ConfigExtractionService: Send + Sync {
    async fn extract_commit_sha(
        &self,
        service_name: &str,
        env: &str,
    ) -> anyhow::Result<ExtractCommitShaResult>;
}

pub struct GithubConfigExtractionServiceImpl {
    github_service: Box<dyn GithubService>,
}

impl GithubConfigExtractionServiceImpl {
    pub fn new(github_service: Box<dyn GithubService>) -> Self {
        GithubConfigExtractionServiceImpl { github_service }
    }
}

#[derive(Debug)]
pub struct ExtractCommitShaResult {
    pub commit_sha: String,
}

#[async_trait]
impl ConfigExtractionService for GithubConfigExtractionServiceImpl {
    async fn extract_commit_sha(
        &self,
        service_name: &str,
        env: &str,
    ) -> anyhow::Result<ExtractCommitShaResult> {
        let owner = "jrumjantsev".into(); // TODO: get owner from the config
        let repo = "config".into();
        let metafile = format!("apps/{}/config.json", service_name);

        let commit_contents = self
            .github_service
            .get_contents(owner, repo, metafile.as_str())
            .await;

        match commit_contents {
            Ok(contents) => {
                let contents_result: Result<serde_json::Value, serde_json::Error> =
                    serde_json::from_str(&contents);
                if contents_result.is_err() {
                    let err = contents_result.err().unwrap();
                    println!("Error parsing JSON: {:?}", err);
                    return Err(anyhow!(err));
                }

                let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
                let path_to_image_tag_value =
                    format!("/service/{}/env/{}/imageTag", service_name, env,);

                let image_tag_value =
                    if let Some(value) = contents_json.pointer(path_to_image_tag_value.as_str()) {
                        value.as_str().unwrap()
                    } else {
                        println!(
                            "The specified path does not exist. Path {}",
                            path_to_image_tag_value
                        );
                        panic!();
                    };

                let commit_sha = image_tag_value.split('-').last().unwrap().to_string();

                Ok(ExtractCommitShaResult { commit_sha })
            }
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}

mod tests {
    use super::*;
    use crate::api::github_service::MockGithubService;
    use futures::FutureExt;
    use mockall::predicate::*;

    #[tokio::test]
    async fn unit_test_extract_commit_sha() {
        let response_payload = r#"{"service": {"foo": {"env": {"dev": {"imageTag": "foo123"}}}}}"#;
        let mut github_service = MockGithubService::new();
        github_service
            .expect_get_contents()
            .with(
                eq("jrumjantsev".to_string()),
                eq("config".to_string()),
                eq("apps/foo/config.json"),
            )
            .times(1)
            .returning(move |_, _, _| {
                let response_payload = response_payload.to_string();
                async move { Ok(response_payload) }.boxed()
            });

        let service = GithubConfigExtractionServiceImpl::new(Box::new(github_service));
        let result = service.extract_commit_sha("foo", "dev").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().commit_sha, "foo123");
    }
}
