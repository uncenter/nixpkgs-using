use clap::{ArgAction, Parser, ValueEnum};
use color_eyre::eyre::{bail, eyre, ContextCompat, Ok, Result};
use nixpkgs_using::{detect_configuration, eval_nix_configuration};

use std::env;
use std::process::Command;

use serde::Serialize;
use users::get_current_username;

mod github;
use github::paginate_pull_requests;

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
#[command(version, about)]
struct Cli {
	// GitHub token
	#[clap(long, short)]
	token: Option<String>,

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

	// See https://jwodder.github.io/kbits/posts/clap-bool-negate/.
	// Cursed code to enable the correct relationship between `--home-manager-packages` and `--no-home-manager-packages`.
	/// Enable searching through Home Manager packages
	#[clap(long = "home-manager-packages", overrides_with = "home_manager_packages")]
	_no_home_manager_packages: bool,
	/// Disable searching through Home Manager packages
	#[clap(long = "no-home-manager-packages", action = ArgAction::SetFalse)]
	home_manager_packages: bool,
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let token: String = match args.token {
		Some(value) => value,
		None => env::var("GITHUB_TOKEN").unwrap_or(env::var("GH_TOKEN").or_else(|_| bail!("No GitHub token provided and not found in `GITHUB_TOKEN`/`GH_TOKEN` environment variables"))?),
	};

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

	let packages = eval_nix_configuration(flake, configuration, username, args.home_manager_packages);
	let prs = paginate_pull_requests(owner.to_string(), repo.to_string(), token)?;

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

	match args.output {
		Output::Json => println!("{}", serde_json::to_string(&filtered).unwrap()),
		Output::Table => {
			let mut table = Table::new(&filtered);
			table.with(Style::rounded());

			println!("{}", table.to_string());
		}
	}

	Ok(())
}
