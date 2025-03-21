#![allow(clippy::missing_errors_doc)]
use clap::Parser;
use color_eyre::eyre::{bail, ContextCompat, Ok, Result};
use nixpkgs_using::cli::{Cli, Commands};
use nixpkgs_using::COMMON_EXTRA_PACKAGES;
use nixpkgs_using::{detect_configuration, eval_nix_configuration, get_hostname, github::paginate_pull_requests};
use std::fs;

use yansi::hyperlink::HyperlinkExt;
use yansi::Paint;

use chrono::{TimeZone, Utc};
use etcetera::{choose_base_strategy, BaseStrategy};
use serde::Serialize;
use users::get_current_username;

#[derive(Debug, Serialize)]
struct Entry {
	title: String,
	url: String,
	new: bool,
}

fn main() -> Result<()> {
	let args = Cli::parse();
	let displayln = |message: String| {
		if !args.json {
			println!("{}", message);
		}
	};
	color_eyre::install()?;

	let cache_dir = choose_base_strategy()?
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

	let hostname = get_hostname();
	let configuration = detect_configuration()?;
	let combined_configuration = configuration + "." + hostname.trim();

	displayln(format!(
		"Evaluating user configuration (username: {}, configuration: {})... ",
		username.green(),
		combined_configuration.blue()
	));

	let mut packages = eval_nix_configuration(&args.flake, &combined_configuration, &username, args.system_packages, args.home_manager_packages)?;
	packages.retain(|pkg| !COMMON_EXTRA_PACKAGES.contains(&pkg.as_str()));

	displayln(format!("{} packages detected.\n", packages.len().to_string().yellow()));

	match args.command {
		Commands::Prs {
			token,
			repository,
			only_new,
			only_updates,
		} => {
			let most_recent_pr_store = cache_dir.join("most_recent_pr");
			if !most_recent_pr_store.exists() {
				fs::write(&most_recent_pr_store, "0")?;
			};

			let most_recent_pr = Utc
				.timestamp_opt(fs::read_to_string(&most_recent_pr_store)?.parse::<i64>()?, 0)
				.unwrap();

			let parts = repository
				.split('/')
				.collect::<Vec<_>>();
			let [owner, repo] = parts.as_slice() else {
				bail!("Invalid repository format");
			};

			displayln(format!("Fetching pull requests..."));

			let prs = paginate_pull_requests(owner, repo, &token)?;

			displayln(format!("{} pull requests found.\n", prs.len().to_string().yellow()));

			let filtered: Vec<Entry> = prs
				.iter()
				.flatten()
				.filter_map(|pr| {
					let title_contains_update = !only_updates || pr.title.contains("->");
					let title_has_match = packages.iter().any(|pkg| {
						pr.title
							.starts_with(&(pkg.to_owned() + ":"))
					});

					if !pr.is_draft && title_contains_update && title_has_match {
						let new = pr
							.created_at
							.signed_duration_since(most_recent_pr)
							.num_milliseconds() > 0;

						// Either all are included (not only new ones), or only new ones are included and this one is new.
						if !only_new || new {
							return Some(Entry {
								title: pr.title.clone(),
								url: pr.url.clone(),
								new,
							});
						};
					};

					None
				})
				.collect();

			println!(
				"{}",
				if args.json {
					serde_json::to_string(&filtered)?
				} else {
					format!("{} filtered pull requests.\n", &filtered.len().to_string().magenta())
						+ &filtered
							.iter()
							.map(|pr| {
								let new = if pr.new { " (new)".green().to_string() } else { "".to_string() };
								format!(" * {}{}", pr.title.link(pr.url.clone()), new)
							})
							.collect::<Vec<_>>()
							.join("\n")
				}
			);

			let mut sorted: Vec<_> = prs.into_iter().flatten().collect();
			sorted.sort_by_key(|pr| pr.created_at.timestamp());

			if let Some(latest_pr) = sorted.last() {
				fs::write(
					most_recent_pr_store,
					latest_pr
						.created_at
						.timestamp()
						.to_string(),
				)?;
			}
		}
		Commands::List {} => {
			println!("{}", if args.json { serde_json::to_string(&packages)? } else { packages.join(" ") });
		}
	}

	Ok(())
}
