use crate::api::config_extraction_service::ConfigExtractionService;
use crate::api::github_service::{GetCommitResult, GithubService};
use crate::api::jira_service::{JiraService, JiraTicketInfo};
use crate::api::result_printer_service::{
    PrintMessageFromExtractTicketsResultPayload, ResultPrinterService, TicketInfo,
};
use crate::api::ticket_extraction_service::TicketExtractionService;
use async_trait::async_trait;
use futures::future::try_join_all;
use futures::stream::iter;
use futures::StreamExt;
use std::collections::HashMap;

#[async_trait]
pub trait InfoGatheringService<'a>: Sync + Send {
    fn get_github_service(&self) -> &dyn GithubService;
    fn get_result_printer_service(&self) -> &dyn ResultPrinterService;
    fn get_ticket_extraction_service(&self) -> &dyn TicketExtractionService;
    fn get_jira_service(&self) -> &dyn JiraService;
    fn get_config_extraction_service(&self) -> &dyn ConfigExtractionService;

    async fn show_undeployed_commits(
        &self,
        owner_name: &str,
        service_name: &str,
        env: &str,
    ) -> anyhow::Result<Vec<String>> {
        /*
           Part 1. Extract the commit sha for the service
        */
        let source_sha = self
            .get_config_extraction_service()
            .extract_commit_sha(service_name, env)
            .await?
            .commit_sha;

        /*
           Part 2. Get the latest commit from the service using extracted sha. We're mostly interested in date.
        */
        let get_commit_result = self
            .get_github_service()
            .get_commit(owner_name, service_name, source_sha.as_str(), false)
            .await?;

        /*
           Part 3. Get a list of commits since the date of the latest commit
        */
        let date_time = get_commit_result.date_time;
        let commit_shas = &self
            .get_github_service()
            .get_commits_since(owner_name, service_name, date_time)
            .await?
            .commit_shas;

        /*
           Part 4. Collect additional information for each and every commit
        */
        let all_commits_futures = commit_shas.iter().map(|sha| {
            self.get_github_service()
                .get_commit(owner_name, service_name, sha, true)
        });
        let all_commits = try_join_all(all_commits_futures).await?;

        /*
           Part 5. Working with JIRA tickets
                - Extract JIRA ticket information from the commit messages
                - Query JIRA ticket statuses (Go vs No-Go)
        */
        // TODO: abstract away from here
        // key - JIRA ticket key
        let issue_keys: HashMap<String, Vec<JiraTicketInfo>> = iter(&all_commits)
            .then(|commit: &GetCommitResult| async move {
                for field in vec![&commit.full_message, &commit.pr_title, &commit.pr_body] {
                    let tickets = self.get_ticket_extraction_service().extract_tickets(field);
                    if tickets.is_empty() {
                        continue;
                    }

                    let jira_tickets: Result<Vec<JiraTicketInfo>, String> = self
                        .get_jira_service()
                        .get_jira_issues(tickets.clone())
                        .await;

                    if let Err(err) = jira_tickets {
                        eprintln!("Error fetching JIRA issues: {:?}", err);
                        continue;
                    }

                    return Ok((commit.sha.to_string(), jira_tickets.unwrap()));
                }

                Ok((commit.sha.to_string(), vec![])) // No tickets found in any field
            })
            .filter_map(
                |result: Result<(String, Vec<JiraTicketInfo>), String>| async move {
                    match result {
                        Ok(value) => Some(value),
                        Err(_) => None,
                    }
                },
            )
            .collect()
            .await;

        /*
         Part 6. Group everything by commit sha
        */
        let mut commit_jira_tickets: HashMap<String, JiraTicketShort> = HashMap::new();
        for (commit_sha, jira_tickets) in issue_keys.iter() {
            for ticket in jira_tickets {
                commit_jira_tickets.insert(
                    commit_sha.clone(),
                    JiraTicketShort {
                        ticket_key: ticket.key.clone(),
                        ticket_ready: ticket.ready.clone(),
                    },
                );
            }
        }

        let mut ticket_infos: Vec<TicketInfo> = vec![];
        for commit in all_commits.iter() {
            let jira_info = commit_jira_tickets
                .get(&commit.sha)
                .cloned()
                .unwrap_or_default();

            ticket_infos.push(TicketInfo {
                commit_sha: commit.sha.as_str(),
                author_email: commit.author_email.as_str(),
                commit_message: commit.pr_title.as_str(),
                ticket_key: jira_info.ticket_key.clone(),
                ticket_ready: jira_info.ticket_ready.clone(),
            });
        }

        /*
           Part 7. Print the result
        */
        let output = self
            .get_result_printer_service()
            .print_message_from_extract_tickets_result(
                PrintMessageFromExtractTicketsResultPayload {
                    owner: owner_name,
                    repo: service_name,
                    ticket_infos,
                    issue_keys,
                    last_commit_in_production: get_commit_result.sha,
                    commit_sha_to_release: None,
                },
            );

        Ok(output)
    }
}

#[derive(Clone, Default)]
struct JiraTicketShort {
    ticket_key: String,
    ticket_ready: bool,
}
