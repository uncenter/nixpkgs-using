use color_eyre::eyre::{bail, Result};

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

pub fn eval_nix_configuration(flake: &str, configuration: &str, username: &str, use_system_packages: bool, use_home_manager_packages: bool) -> Result<Vec<String>> {
	let expr = format!(
		"{}{}",
		if use_system_packages {
			format!("(builtins.getFlake \"{flake}\").{configuration}.config.environment.systemPackages")
		} else {
			"[]".to_owned()
		},
		if use_home_manager_packages {
			format!(" ++ (builtins.getFlake \"{flake}\").{configuration}.config.home-manager.users.{username}.home.packages")
		} else {
			String::new()
		}
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

	String::from_utf8(if output.status.success() { output.stdout } else { output.stderr }).unwrap()
}

pub const COMMON_EXTRA_PACKAGES: [&str; 13] = [
	"nix",
	"nix-info",
	// home-manager
	"home-configuration-reference-manpage",
	"hm-session-vars.sh",
	"dummy-fc-dir1", // fontconfig module
	"dummy-fc-dir2",
	// darwin
	"darwin-uninstaller",
	"darwin-version",
	"darwin-rebuild",
	"darwin-option",
	"darwin-manpages",
	"darwin-manual-html",
	"darwin-help",
];
