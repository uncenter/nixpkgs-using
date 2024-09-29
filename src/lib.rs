use color_eyre::eyre::{bail, Result};
use github::{build_rest_request, BASE_API_URL};
use reqwest::StatusCode;
use serde::Deserialize;

use std::{env, path::Path, process::Command};

pub mod cli;
pub mod github;

pub fn detect_configuration() -> Result<String> {
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

pub fn eval_nix_configuration(flake: String, configuration: String, username: String, use_home_manager_packages: bool) -> Result<Vec<String>> {
	let expr = format!(
		"(builtins.getFlake \"{flake}\").{configuration}.config.environment.systemPackages{}",
		(if use_home_manager_packages {
			format!(" ++ (builtins.getFlake \"{flake}\").{configuration}.config.home-manager.users.{username}.home.packages")
		} else {
			String::new()
		})
	);

	let cmd = Command::new("nix")
		.args(["eval", "--impure", "--json", "--expr", &expr, "--apply", "map (pkg: (builtins.parseDrvName pkg.name).name)"])
		.output()
		.expect("failed to execute process");

	if cmd.status.success() {
		Ok(serde_json::from_str(&String::from_utf8(cmd.stdout)?)?)
	} else {
		bail!("unable to evaluate nix configuration: {}", String::from_utf8(cmd.stderr)?);
	}
}

pub fn get_hostname() -> String {
	let output = Command::new("hostname")
		.arg("-s")
		.output()
		.expect("hostname should exist");

	return String::from_utf8(match output.status.success() {
		true => output.stdout,
		false => output.stderr,
	})
	.unwrap();
}

pub fn check_by_name(package: &str, token: &str) -> Result<bool> {
	// https://api.github.com/repos/nixos/nixpkgs/contents/pkgs/by-name/ki/kittysay/package.nix
	let url = format!(
		"{}/contents/pkgs/by-name/{}/{}/package.nix",
		BASE_API_URL,
		package
			.chars()
			.take(2)
			.collect::<String>(),
		package
	);
	let response = build_rest_request(&url, token).send()?;

	match response.status() {
		StatusCode::OK => Ok(true),
		StatusCode::NOT_FOUND => Ok(false),
		_ => bail!("Unexpected response"),
	}
}
