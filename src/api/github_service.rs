use crate::api::errors::{GitHubBaseUrlUndefined, GitHubTokenUndefined};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use octocrab::models::repos::RepoCommit;
use octocrab::{Octocrab, Page};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug)]
pub struct GetCommitResult {
    pub date_time: DateTime<Utc>,
    pub author_email: String,
    pub sha: String,
    pub full_message: String,
    pub pr_title: String,
    pub pr_body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    commit: CommitDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommitDetail {
    committer: Committer,
    author: Author,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Committer {
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    email: String,
}

#[derive(Clone)]
pub struct GetCommitsSinceResult {
    pub commit_shas: Vec<String>,
}

#[async_trait]
pub trait GithubService: Sync + Send {
    /// Get the commit details
    ///
    /// Details include:
    /// - Date-time
    /// - Author email
    /// - SHA
    /// - Full message
    /// - PR title (if `with_pr` is true, additional API request)
    /// - PR body (if `with_pr` is true, additional API request)
    async fn get_commit(
        &self,
        owner_name: &str,
        repo_name: &str,
        commit_sha: &str,
        with_pr: bool,
    ) -> anyhow::Result<GetCommitResult>;

    /// Get the commits since a given date-time
    ///
    /// Returns a list of commit SHAs
    async fn get_commits_since(
        &self,
        owner_name: &str,
        repo_name: &str,
        date_time: DateTime<Utc>,
    ) -> anyhow::Result<GetCommitsSinceResult>;

    /// Get the contents of a file in a repository
    /// Returns the decoded content of the file
    ///
    /// If the file does not exist, returns an error
    /// If the file is not a valid UTF-8 string, returns an error
    /// If the file is not base64 encoded, returns an error
    /// If the file is not a valid JSON, returns an error
    async fn get_contents(
        &self,
        owner_name: &str,
        repo_name: &str,
        file_path: &str,
    ) -> anyhow::Result<String>;

    /// Find the first PR of a commit
    /// Returns the title and body of the PR
    /// If no PR is found, returns empty strings
    async fn find_first_pr_of_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> anyhow::Result<FindFirstPrOfCommitResult>;
}

#[derive(Clone)]
pub struct GithubServiceImpl {
    pub gh: Octocrab,
    pub base_url: String,
    pub token: String,
}

pub struct FindFirstPrOfCommitResult {
    pr_title: String,
    pr_body: String,
}

impl GithubServiceImpl {
    pub fn new(base_url: Option<&str>, token: Option<&str>) -> Result<Self, Box<dyn Error>> {
        match (base_url, token) {
            (None, _) => Err(Box::new(GitHubBaseUrlUndefined)),
            (_, None) => Err(Box::new(GitHubTokenUndefined)),
            (Some(base_url), Some(token)) => {
                let octocrab = Octocrab::builder()
                    .base_uri(base_url)?
                    .personal_token(token.to_string())
                    .build()?;
                Ok(Self {
                    gh: octocrab,
                    base_url: base_url.to_string(),
                    token: token.to_string(),
                })
            }
        }
    }
}

#[async_trait]
impl GithubService for GithubServiceImpl {
    async fn get_commit(
        &self,
        owner_name: &str,
        repo_name: &str,
        commit_sha: &str,
        with_pr: bool,
    ) -> anyhow::Result<GetCommitResult> {
        let response = reqwest::Client::new()
            .get(format!(
                "{}/repos/{}/{}/commits/{}",
                &self.base_url,
                owner_name,
                repo_name,
                commit_sha.to_string()
            ))
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !(response.status().is_success()) {
            return Err(anyhow!("Error"));
        }

        let res_text = response.text().await;
        if res_text.is_err() {
            return Err(anyhow!("Error"));
        }

        let body = res_text.unwrap();
        let parsed_data: Commit = serde_json::from_str(&body)?;

        // Extract the full ISO 8601 date-time string
        let full_date_time = &parsed_data.commit.committer.date;
        let author_email = parsed_data.commit.author.email;
        let message = parsed_data.commit.message;
        let date_time: DateTime<Utc> = full_date_time.parse().expect("Failed to parse date-time");

        // TODO: execute in parallel with previous request
        let (pr_title, pr_body) = if with_pr {
            match self
                .find_first_pr_of_commit(&owner_name, &repo_name, &commit_sha)
                .await
            {
                Ok(pr_info) => (pr_info.pr_title, pr_info.pr_body),
                Err(_) => return Err(anyhow!("Error")),
            }
        } else {
            ("".to_string(), "".to_string())
        };

        return Ok(GetCommitResult {
            date_time: date_time.clone(),
            author_email: author_email.to_string(),
            sha: commit_sha.to_string(),
            full_message: message.to_string(),
            pr_title,
            pr_body,
        });
    }

    async fn get_commits_since(
        &self,
        owner_name: &str,
        repo_name: &str,
        date_time: DateTime<Utc>,
    ) -> anyhow::Result<GetCommitsSinceResult> {
        let commits: Page<RepoCommit> = self
            .gh
            .repos(owner_name, repo_name)
            .list_commits()
            .since(date_time)
            .send()
            .await?;

        let commit_shas: Vec<String> = commits
            .into_iter()
            .map(|commit| commit.sha.clone())
            .collect();

        Ok(GetCommitsSinceResult { commit_shas })
    }

    async fn get_contents(
        &self,
        owner_name: &str,
        repo_name: &str,
        file_path: &str,
    ) -> anyhow::Result<String> {
        let contents = self
            .gh
            .repos(owner_name, repo_name)
            .get_content()
            .path(file_path)
            .send()
            .await?;

        return Ok(contents.items[0].decoded_content().unwrap());
    }

    async fn find_first_pr_of_commit(
        &self,
        owner: &str,
        repo: &str,
        commit_sha: &str,
    ) -> anyhow::Result<FindFirstPrOfCommitResult> {
        let response = reqwest::Client::new()
            .get(format!(
                "{}/search/issues?q=SHA:{}+repo:{}/{}+type:pr&sort=created&order=asc",
                &self.base_url, commit_sha, owner, repo
            ))
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !(response.status().is_success()) {
            println!("Error: {}", response.status());
            return Err(anyhow!("Error"));
        }

        let body = response.text().await?;
        let parsed_data: serde_json::Value = serde_json::from_str(&body)?;

        // check if items exists and not empty
        if parsed_data["items"].is_null() || parsed_data["items"].as_array().unwrap().is_empty() {
            return Ok(FindFirstPrOfCommitResult {
                pr_title: "".to_string(),
                pr_body: "".to_string(),
            });
        }

        let pr_title = parsed_data["items"][0]["title"].as_str().unwrap_or("");
        let pr_body = parsed_data["items"][0]["body"].as_str().unwrap_or("");

        Ok(FindFirstPrOfCommitResult {
            pr_title: pr_title.to_string(),
            pr_body: pr_body.to_string(),
        })
    }
}
