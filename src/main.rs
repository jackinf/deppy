mod cli;
mod config;
mod constants;

mod api {
    pub mod config_extraction_service;
    pub mod errors;
    pub mod github_service;
    pub mod jira_service;
    pub mod result_printer_service;
    pub mod ticket_extraction_service;
    pub mod version_service;
}
mod domain {
    pub mod to_deploy {
        pub mod services {
            pub mod bar_info_gathering_service;
            pub mod foo_info_gathering_service;
            pub mod info_gathering_service;
        }
    }
    pub mod doctor {
        pub mod doctor_app;
    }
}

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::domain::to_deploy::services::bar_info_gathering_service::BarInfoGatheringService;
use crate::domain::to_deploy::services::foo_info_gathering_service::FooInfoGatheringService;
use crate::domain::to_deploy::services::info_gathering_service::InfoGatheringService;
use clap::Parser;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // Load .env file
    env_logger::init(); // Initialize logger

    let cli = Cli::parse();

    match cli.command {
        Commands::ToDeploy(args) => match args.project.as_str() {
            constants::PROJECT_FOO_WEB => {
                let _ = FooInfoGatheringService::new(&Config::new())
                    .show_undeployed_commits("MainAppServices", args.project.as_str(), "prod")
                    .await;
            }
            constants::PROJECT_BAR_WEB => {
                let _ = BarInfoGatheringService::new(&Config::new())
                    .show_undeployed_commits("MainAppServices", args.project.as_str(), "prod")
                    .await;
            }
            // Add more projects if needed
            _ => println!("Project not found: {}", args.project),
        },
        Commands::Doctor(_) => {
            todo!("Another application logic here that will be implemented in the future for analyzing JIRA tickets that are in the wrong status.")
        }
    }
}
