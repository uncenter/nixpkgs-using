use clap::{Parser, ValueEnum};
use color_eyre::eyre::{bail, eyre, ContextCompat, Ok, Result};

use std::env;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use users::get_current_username;

use tabled::settings::Style;
use tabled::{Table, Tabled};

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
struct Cli {
	#[clap(long, short)]
	flake: Option<String>,
	#[clap(long, short)]
	configuration: Option<String>,
	#[clap(long, short)]
	username: Option<String>,
	#[clap(long, short, default_value = "nixos/nixpkgs")]
	repository: String,
	#[clap(long, short, value_enum, default_value = "table")]
	output: Output,
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequest {
	title: String,
	number: u64,
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

#[tokio::main]
async fn main() -> Result<()> {
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

	let prs: Vec<PullRequest> = serde_json::from_str(
		String::from_utf8(
			Command::new("gh")
				.args(["pr", "list", format!("--repo={}", args.repository).as_str(), "--draft=false", "--limit=5000", "--json", "title,number"])
				.output()
				.unwrap()
				.stdout,
		)
		.unwrap()
		.as_str(),
	)
	.unwrap();

	let filtered = prs
		.into_iter()
		.filter_map(|pr| {
			let title = pr.title;
			let number = pr.number;

			if packages
				.clone()
				.into_iter()
				.any(|pkg| title.starts_with(&(pkg + ":")))
			{
				Some(Entry {
					title,
					url: format!("https://github.com/{owner}/{repo}/pull/{number}"),
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
