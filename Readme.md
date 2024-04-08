# Deppy Rust

## Description

App is designed to collect information about commits, extract JIRA tickets from commit titles & messages, use these JIRA 
tickets to get information about deployments, and then return a list of commits that have not been deployed to a specific
environment.

It implies that the JIRA ticket carry the information if the functionality has been deployed or not.

## Setup

To set up the project, follow these steps:

1. Install Rust and Cargo if you haven't already.
2. Clone the repository.
3. Run `cargo build` to build the project.
4. Run `cargo run` to start the project.

## Usage

The project can be used via the command line interface. Here are some example commands:

- To return all commits not deployed to Foo prod: `make foo-web-prod`
- To return all commits not deployed to Foo staging: `make foo-web-staging`
- To return all commits not deployed to Bar prod: `make bar-web-prod`
- To return all commits not deployed to Bar staging: `make bar-web-staging`

For a full list of commands, refer to the `Makefile`.

## Contributing

Contributions are welcome. Please submit a pull request or create an issue to discuss the changes you want to make.

## License

This project is licensed under the MIT License.