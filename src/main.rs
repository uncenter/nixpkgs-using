use clap::{ArgAction, Parser, ValueEnum};
use color_eyre::eyre::{bail, ContextCompat, Ok, Result};
use nixpkgs_using::{detect_configuration, eval_nix_configuration, get_hostname};

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
	#[clap(long, short, env = "GITHUB_TOKEN")]
	token: String,

	/// Path to the flake to evaluate
	#[clap(long, short, env = "FLAKE")]
	flake: String,
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

	/// Exclude pull requests that are not updating a package
	#[clap(long)]
	only_updates: bool,
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let username: String = match args.username {
		Some(value) => value,
		None => get_current_username()
			.context("Failed to get current username")?
			.into_string()
			.unwrap(),
	};

	let configuration = (detect_configuration()? + "." + &get_hostname())
		.trim()
		.to_string();

	let parts = args
		.repository
		.split('/')
		.collect::<Vec<_>>();
	let [owner, repo] = parts.as_slice() else {
		bail!("Invalid repository format");
	};

	let packages = eval_nix_configuration(args.flake, configuration, username, args.home_manager_packages);
	let prs = paginate_pull_requests(owner.to_string(), repo.to_string(), args.token)?;

	let filtered = prs
		.iter()
		.flatten()
		.filter(|pr| {
			let is_draft = pr.is_draft;
			let title_contains_update = !args.only_updates || pr.title.contains("->");
			!is_draft
				&& title_contains_update
				&& packages.iter().any(|pkg| {
					pr.title
						.starts_with(&(pkg.to_owned() + ":"))
				})
		})
		.map(|pr| Entry {
			title: pr.title.clone(),
			url: pr.url.clone(),
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
