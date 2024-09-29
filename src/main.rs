use clap::Parser;
use color_eyre::eyre::{bail, ContextCompat, Ok, Result};
use nixpkgs_using::check_by_name;
use nixpkgs_using::cli::{Cli, Commands, Output};
use nixpkgs_using::{detect_configuration, eval_nix_configuration, get_hostname, github::paginate_pull_requests};
use std::fs;

use chrono::{TimeZone, Utc};
use etcetera::{choose_base_strategy, BaseStrategy};
use serde::Serialize;
use users::get_current_username;

use tabled::settings::{location::ByColumnName, object::Rows, style::BorderSpanCorrection, themes::Colorization, Color, Disable, Panel, Style};
use tabled::{Table, Tabled};

#[derive(Tabled, Debug, Serialize)]
#[tabled(rename_all = "PascalCase")]
struct Entry {
	title: String,
	#[tabled(rename = "URL")]
	url: String,
	new: bool,
}

fn main() -> Result<()> {
	let args = Cli::parse();
	color_eyre::install()?;

	let cache_dir = choose_base_strategy()
		.unwrap()
		.cache_dir()
		.join("nixpkgs-using");
	if !cache_dir.exists() {
		fs::create_dir_all(&cache_dir)?;
	}

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

	let packages = eval_nix_configuration(args.flake, configuration, username, args.home_manager_packages)?;

	match args.command {
		Commands::Prs { only_new, only_updates } => {
			let most_recent_pr_store = cache_dir.join("most_recent_pr");
			if !most_recent_pr_store.exists() {
				fs::write(&most_recent_pr_store, "0").unwrap();
			};

			let most_recent_pr = Utc
				.timestamp_opt(
					fs::read_to_string(&most_recent_pr_store)
						.unwrap()
						.parse::<i64>()
						.unwrap(),
					0,
				)
				.unwrap();

			let prs = paginate_pull_requests(owner.to_string(), repo.to_string(), args.token)?;

			let filtered = prs
				.iter()
				.flatten()
				.filter_map(|pr| {
					let is_draft = pr.is_draft;
					let title_contains_update = !only_updates || pr.title.contains("->");
					let title_has_match = packages.iter().any(|pkg| {
						pr.title
							.starts_with(&(pkg.to_owned() + ":"))
					});

					if !is_draft && title_contains_update && title_has_match {
						let new = pr
							.created_at
							.signed_duration_since(most_recent_pr)
							.num_milliseconds() > 0;

						Some(Entry {
							title: pr.title.clone(),
							url: pr.url.clone(),
							new: new,
						})
					} else {
						None
					}
				})
				.filter(|pr| !only_new || (only_new && pr.new))
				.collect::<Vec<_>>();

			match args.output {
				Output::Json => println!("{}", serde_json::to_string(&filtered).unwrap()),
				Output::Table => {
					if filtered.len() > 0 {
						let mut table = Table::new(&filtered);
						table.with(Style::modern());

						if only_new {
							table.with(Disable::column(ByColumnName::new("New")));
						}

						table
							.with(Panel::footer(format!("{} pull requests found.", filtered.len())))
							.with(Colorization::exact([Color::BOLD], Rows::last()))
							.with(Colorization::exact([Color::BOLD], Rows::first()))
							.with(BorderSpanCorrection);

						println!("{}", table.to_string());
					} else {
						println!("0 pull requests found.")
					}
				}
			}

			let mut sorted: Vec<_> = prs.into_iter().flatten().collect();
			sorted.sort_by_key(|pr| pr.created_at.timestamp());

			if let Some(latest_pr) = sorted.last() {
				fs::write(
					most_recent_pr_store,
					latest_pr
						.created_at
						.timestamp()
						.to_string(),
				)
				.expect("Unable to write file");
			}
		}
		Commands::List {} => {
			println!("{}", serde_json::to_string(&packages).unwrap());
		}
		Commands::Check {} => {
			for package in packages {
				let by_name = check_by_name(&package, &args.token)?;
				if !by_name {
					println!("{}: is not located in `pkgs/by-name/`", package);
				}
			}
		}
	}

	Ok(())
}
