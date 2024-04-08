use crate::api::jira_service::JiraTicketInfo;
use crate::config::Config;
use std::collections::HashMap;
use std::option::Option;

#[derive(Debug, Clone)]
pub struct TicketInfo<'a> {
    pub commit_sha: &'a str,
    pub commit_message: &'a str,
    pub author_email: &'a str,
    pub ticket_key: String,
    pub ticket_ready: bool,
}

#[derive(Debug)]
pub struct TicketInfoGroup<'a> {
    pub info_items: Vec<TicketInfo<'a>>,
}

#[derive(Debug)]
pub struct PrintMessageFromExtractTicketsResultPayload<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub ticket_infos: Vec<TicketInfo<'a>>,
    pub last_commit_in_production: String,
    pub commit_sha_to_release: Option<String>,
    pub issue_keys: HashMap<String, Vec<JiraTicketInfo>>,
}

pub trait ResultPrinterService: Sync + Send {
    fn print_message_from_extract_tickets_result(
        &self,
        payload: PrintMessageFromExtractTicketsResultPayload,
    );
}

pub struct ResultPrinterServiceImpl<'a> {
    pub config: &'a Config,
}

impl<'a> ResultPrinterServiceImpl<'a> {
    pub fn new(config: &'a Config) -> Self {
        ResultPrinterServiceImpl { config: &config }
    }
}

impl<'a> ResultPrinterService for ResultPrinterServiceImpl<'a> {
    fn print_message_from_extract_tickets_result(
        &self,
        payload: PrintMessageFromExtractTicketsResultPayload,
    ) {
        let owner = &payload.owner;
        let repo = &payload.repo;
        let ticket_infos = &payload.ticket_infos;
        let last_commit_in_production = &payload.last_commit_in_production;
        let commit_sha_to_release = payload.commit_sha_to_release.as_deref().unwrap_or("master");

        println!(
            "{}/{}/{}/compare/{}...{}\n",
            self.config.github_server,
            owner,
            repo,
            last_commit_in_production,
            commit_sha_to_release
        );

        ticket_infos.iter().for_each(|commit| {
            let author_username = commit.author_email.split('@').next().unwrap_or("");
            let ticket_ready_icon = if commit.ticket_ready { "🍏" } else { "🍎" };
            let short_commit_sha = &commit.commit_sha[..7.min(commit.commit_sha.len())];

            println!(
                "{} @{} {}/{}/{}/commit/{} ({}) - [{}] {}",
                ticket_ready_icon,
                author_username,
                self.config.github_server,
                owner,
                repo,
                commit.commit_sha,
                short_commit_sha,
                commit.ticket_key,
                commit.commit_message
            );
        });
    }
}
