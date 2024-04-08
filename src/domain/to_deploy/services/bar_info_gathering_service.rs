use crate::api::config_extraction_service::{
    ConfigExtractionService, GithubConfigExtractionServiceImpl,
};
use crate::api::github_service::{GithubService, GithubServiceImpl};
use crate::api::jira_service::{JiraService, JiraServiceImpl};
use crate::api::result_printer_service::{ResultPrinterService, ResultPrinterServiceImpl};
use crate::api::ticket_extraction_service::{BarTicketExtractionService, TicketExtractionService};
use crate::config::Config;
use crate::domain::to_deploy::services::info_gathering_service::InfoGatheringService;
use async_trait::async_trait;

pub struct BarInfoGatheringService<'a> {
    github_service: Box<dyn GithubService + 'a>,
    ticket_extraction_service: Box<dyn TicketExtractionService + 'a>,
    result_printer_service: Box<dyn ResultPrinterService + 'a>,
    jira_service: Box<dyn JiraService + 'a>,
    config_extraction_service: Box<dyn ConfigExtractionService + 'a>,
}

impl<'a> BarInfoGatheringService<'a> {
    pub fn new(config: &'a Config) -> Self
    where
        Self: Sized,
    {
        let github_service = GithubServiceImpl::new(
            Some(&config.github_server.as_str()),
            Some(&config.github_token.as_str()),
        )
        .unwrap();

        BarInfoGatheringService {
            github_service: Box::new(github_service.clone()),
            ticket_extraction_service: Box::new(BarTicketExtractionService::new()),
            result_printer_service: Box::new(ResultPrinterServiceImpl::new(config.clone())),
            jira_service: Box::new(JiraServiceImpl::new(
                config.jira_server.to_string(),
                config.jira_token.to_string(),
            )),
            config_extraction_service: Box::new(GithubConfigExtractionServiceImpl::new(Box::new(
                github_service.clone(),
            ))),
        }
    }
}

#[async_trait]
impl<'a> InfoGatheringService<'a> for BarInfoGatheringService<'a> {
    fn get_github_service(&self) -> &dyn GithubService {
        self.github_service.as_ref()
    }

    fn get_result_printer_service(&self) -> &dyn ResultPrinterService {
        self.result_printer_service.as_ref()
    }

    fn get_ticket_extraction_service(&self) -> &dyn TicketExtractionService {
        self.ticket_extraction_service.as_ref()
    }

    fn get_jira_service(&self) -> &dyn JiraService {
        self.jira_service.as_ref()
    }

    fn get_config_extraction_service(&self) -> &dyn ConfigExtractionService {
        self.config_extraction_service.as_ref()
    }
}
