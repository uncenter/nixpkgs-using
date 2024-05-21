# nixpkgs-using

Find packages that you use that are currently being updated in Nixpkgs.

```
╭───────────────────────────────────┬──────────────────────────────────────────────╮
│ title                             │ url                                          │
├───────────────────────────────────┼──────────────────────────────────────────────┤
│ ruff: 0.4.3 -> 0.4.4              │ https://github.com/nixos/nixpkgs/pull/310440 │
│ vscode: 1.89.0 -> 1.89.1          │ https://github.com/nixos/nixpkgs/pull/310396 │
│ spicetify-cli: rename bin         │ https://github.com/nixos/nixpkgs/pull/309570 │
│ git: 2.44.0 -> 2.45.0             │ https://github.com/nixos/nixpkgs/pull/308186 │
│ imagemagick: 7.1.1-30 -> 7.1.1-32 │ https://github.com/nixos/nixpkgs/pull/307309 │
╰───────────────────────────────────┴──────────────────────────────────────────────╯
```

## Installation

### Cargo

```sh
cargo install --git https://github.com/uncenter/nixpkgs-using.git
```

### Nix

```
nix run github:uncenter/nixpkgs-using
```

## Usage

```
nixpkgs-using [OPTIONS]
```

Requires a GitHub API token to use (provided through the `--token` flag or from the `GITHUB_TOKEN`/`GH_TOKEN` environment variables). With roughly 6,000 open PRs on [NixOS/nixpkgs](https://github.com/NixOS/nixpkgs), ~60 API requests are made per run. GitHub's documentation on GraphQL ratelimiting isn't very clear so I can't say for certain how many runs it will take for the rate limit to be reached, but for good measure don't run it more then 5-ish times an hour.

### `--flake`

Path to the flake to evaluate. Defaults to the `FLAKE` environment variable, if present.

### `--configuration`

Configuration to extract packages from (e.g. `darwinConfigurations.Katara`). Defaults to `*configurations.<hostname>`, where the `*configurations*` is detected from your operating system and the presence of `/etc/NIXOS`.

### `--username`

Username to locate [Home Manager](https://github.com/nix-community/home-manager) packages from.

### `--repository`

The (GitHub) repository from which pull requests are fetched. Defaults to [`NixOS/nixpkgs`](https://github.com/NixOS/nixpkgs).

### `--output`

Output format for the results of the search. One of `json` or `table`. Defaults to `table`.

### `--only-updates`

Exclude pull requests that are not updating a package (filters for pull requests that include a `->` in the title, as is the preferred commit/pull request title style in [NixOS/nixpkgs](https://github.com/NixOS/nixpkgs)).

### `--only-new`

Exclude pull requests that have already been shown by the tool (filters for pull requests created after the timestamp of the last `nixpkgs-using` invocation).

## License

[MIT](LICENSE)
