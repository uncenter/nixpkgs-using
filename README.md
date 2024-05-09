# nixpkgs-using

Find packages that you use that are currently being updated in Nixpkgs.

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

## License

[MIT](LICENSE)
