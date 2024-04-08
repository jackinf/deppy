use std::env;

#[derive(Clone)]
pub struct Config {
    pub github_token: String,
    pub github_server: String,
    pub jira_token: String,
    pub jira_server: String,
    pub versions_live: String,
}

impl Config {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        let (_, github_token) = env::vars()
            .find(|(key, _)| key == "GITHUB_TOKEN")
            .expect("GITHUB_TOKEN not found");
        let (_, github_server) = env::vars()
            .find(|(key, _)| key == "GITHUB_SERVER")
            .expect("failed to get github server");
        let (_, jira_token) = env::vars()
            .find(|(key, _)| key == "JIRA_TOKEN")
            .expect("failed to get jira token");
        let (_, jira_server) = env::vars()
            .find(|(key, _)| key == "JIRA_SERVER")
            .expect("failed to get jira server");
        let (_, versions_live) = env::vars()
            .find(|(key, _)| key == "VERSIONS_URL_LIVE")
            .expect("failed to get versions live server");

        Config {
            github_token,
            github_server,
            jira_token,
            jira_server,
            versions_live,
        }
    }
}
