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

	#[clap(long, default_value = "true", global = true, action = ArgAction::Set)]
	pub home_manager_packages: bool,
	#[clap(long, default_value = "true", global = true, action = ArgAction::Set)]
	pub system_packages: bool,
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
