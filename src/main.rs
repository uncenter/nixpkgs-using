use clap::{Parser, ValueEnum};
use color_eyre::eyre::{bail, eyre, ContextCompat, Ok, Result};

use std::env;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use users::get_current_username;

use tabled::settings::Style;
use tabled::{Table, Tabled};

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

#[allow(clippy::upper_case_acronyms)]
type URI = String;
#[derive(GraphQLQuery)]
#[graphql(schema_path = "src/schema.graphql", query_path = "src/query.graphql", response_derives = "Debug")]
struct PullRequests;
use crate::pull_requests::{PullRequestsRepositoryPullRequests, PullRequestsRepositoryPullRequestsNodes};

#[derive(Tabled, Debug, Serialize)]
struct Entry {
	title: String,
	url: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Output {
	Json,
	Table,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
	#[clap(long, short)]
	token: String,
	/// Path to the flake to evaluate
	#[clap(long, short)]
	flake: Option<String>,
	/// Configuration to extract packages from
	#[clap(long, short)]
	configuration: Option<String>,
	/// Username to locate Home Manager packages from
	#[clap(long, short)]
	username: Option<String>,
	/// The (GitHub) repository from which pull requests are fetched
	#[clap(long, short, default_value = "nixos/nixpkgs")]
	repository: String,
	/// Output format for the results of the search
	#[clap(long, short, value_enum, default_value = "table")]
	output: Output,
}

fn detect_configuration() -> Result<String> {
	match env::consts::OS {
		"linux" => {
			if Path::new("/etc/NIXOS").exists() {
				Ok("nixosConfigurations".to_string())
			} else {
				Ok("homeConfigurations".to_string())
			}
		}
		"macos" => Ok("darwinConfigurations".to_string()),
		_ => bail!("Unsupported operating system detected"),
	}
}

fn get_pull_requests(client: Client, owner: String, repo: String, cursor: std::option::Option<std::string::String>) -> PullRequestsRepositoryPullRequests {
	let variables = pull_requests::Variables {
		owner: owner.to_string(),
		name: repo.to_string(),
		cursor: cursor,
	};

	let response_body = post_graphql::<PullRequests, _>(&client, "https://api.github.com/graphql", variables).unwrap();

	let response_data: pull_requests::ResponseData = response_body
		.data
		.expect("missing response data");

	return response_data
		.repository
		.expect("missing repository")
		.pull_requests;
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let flake: String = match args.flake {
		Some(value) => value,
		None => env::var("FLAKE").or_else(|_| Err(eyre!("No flake path provided and `FLAKE` environment variable not found")))?,
	};

	let username: String = match args.username {
		Some(value) => value,
		None => get_current_username()
			.context("Failed to get current username")?
			.into_string()
			.unwrap(),
	};

	let configuration = detect_configuration().unwrap()
		+ "." + String::from_utf8(
		Command::new("hostname")
			.arg("-s")
			.output()
			.or_else(|_| Err(eyre!("Unable to detect hostname using `hostname` command")))?
			.stdout,
	)
	.unwrap()
	.as_str()
	.trim();

	let parts = args
		.repository
		.split('/')
		.collect::<Vec<_>>();
	let [owner, repo] = parts.as_slice() else {
		bail!("Invalid repository format");
	};

	let packages: Vec<String> = serde_json::from_str(
		String::from_utf8(
			Command::new("nix")
				.args([
					"eval",
					"--impure",
					"--json",
					"--expr",
					format!(
						"(builtins.getFlake \"{flake}\").{configuration}.config.home-manager.users.{username}.home.packages ++ (builtins.getFlake \"{flake}\").{configuration}.config.environment.systemPackages",
					)
					.as_str(),
					"--apply",
					"map (pkg: (builtins.parseDrvName pkg.name).name)",
				])
				.output()
				.unwrap()
				.stdout,
		)
		.unwrap()
		.as_str(),
	)
	.unwrap();

	let client = Client::builder()
		.user_agent("graphql-rust/0.10.0")
		.default_headers(std::iter::once((reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {}", args.token)).unwrap())).collect())
		.build()?;

	let mut cursor = None;
	let mut prs: Vec<Option<PullRequestsRepositoryPullRequestsNodes>> = vec![];

	loop {
		let data = get_pull_requests(client.clone(), owner.to_string(), repo.to_string(), cursor);

		prs.extend(
			data.nodes
				.expect("pull requests nodes is null"),
		);

		if !data.page_info.has_next_page {
			break;
		}
		cursor = data.page_info.end_cursor;
	}

	let filtered = prs
		.iter()
		.flatten()
		.filter_map(|pr| {
			let title = &pr.title;
			let url = &pr.url;
			let is_draft = &pr.is_draft;

			if !is_draft
				&& packages
					.clone()
					.into_iter()
					.any(|pkg| (title).starts_with(&(pkg + ":")))
			{
				Some(Entry {
					title: title.to_string(),
					url: url.to_string(),
				})
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	if args.output == Output::Table {
		let mut table = Table::new(&filtered);
		table.with(Style::rounded());

		println!("{}", table.to_string());
	} else if args.output == Output::Json {
		println!("{}", serde_json::to_string(&filtered).unwrap());
	}

	Ok(())
}
