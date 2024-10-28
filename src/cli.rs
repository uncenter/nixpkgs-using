use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,

	/// Path to the flake to evaluate
	#[clap(long, short, env = "FLAKE")]
	pub flake: String,
	/// Configuration to extract packages from
	#[clap(long, short)]
	pub configuration: Option<String>,
	/// Username to locate Home Manager packages from
	#[clap(long, short)]
	pub username: Option<String>,

	/// Print results in JSON
	#[clap(long, global = true)]
	pub json: bool,

	// See https://jwodder.github.io/kbits/posts/clap-bool-negate/.
	// Cursed code to enable the correct relationship between `--home-manager-packages` and `--no-home-manager-packages`.
	/// Enable searching through Home Manager packages
	#[clap(long = "home-manager-packages", overrides_with = "home_manager_packages")]
	pub _no_home_manager_packages: bool,
	/// Disable searching through Home Manager packages
	#[clap(long = "no-home-manager-packages", action = ArgAction::SetFalse)]
	pub home_manager_packages: bool,
}

#[derive(Subcommand)]
pub enum Commands {
	/// List update pull requests for packages you use
	Prs {
		// GitHub token
		#[clap(long, short, env = "GITHUB_TOKEN")]
		token: String,

		/// The GitHub repository from which pull requests are fetched
		#[clap(long, short, default_value = "nixos/nixpkgs")]
		repository: String,

		/// Exclude pull requests that have already been shown
		#[clap(long)]
		only_new: bool,

		/// Exclude pull requests not detected to be a version update
		#[clap(long)]
		only_updates: bool,
	},
	/// List packages you use
	List {},
}
