use color_eyre::eyre::{bail, Result};

use std::{env, path::Path, process::Command};

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

pub fn eval_nix_configuration(flake: String, configuration: String, username: String, use_home_manager_packages: bool) -> Vec<String> {
	let expr = format!(
		"(builtins.getFlake \"{flake}\").{configuration}.config.environment.systemPackages{}",
		(if use_home_manager_packages {
			format!(" ++ (builtins.getFlake \"{flake}\").{configuration}.config.home-manager.users.{username}.home.packages")
		} else {
			String::new()
		})
	);

	return serde_json::from_str(
		String::from_utf8(
			Command::new("nix")
				.args(["eval", "--impure", "--json", "--expr", &expr, "--apply", "map (pkg: (builtins.parseDrvName pkg.name).name)"])
				.output()
				.unwrap()
				.stdout,
		)
		.unwrap()
		.as_str(),
	)
	.unwrap();
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
