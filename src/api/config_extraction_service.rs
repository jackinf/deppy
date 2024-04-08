use crate::api::github_service::GithubService;
use anyhow::anyhow;
use async_trait::async_trait;

#[async_trait]
pub trait ConfigExtractionService: Send + Sync {
    async fn extract_commit_sha(
        &self,
        service_name: &str,
        service_sub_name: &str,
        env: &str,
        cluster: Option<&str>,
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
        service_sub_name: &str,
        env: &str,
        cluster: Option<&str>,
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
                let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
                let path_to_image_tag_value = format!(
                    "/spec/workloads/{}/clusters/{}/envs/{}/tracks/main/containers/{}/imageTag",
                    service_name,
                    cluster.unwrap(),
                    env,
                    service_sub_name
                );

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
