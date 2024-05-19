{
  lib,
  darwin,
  stdenv,
  openssl,
  pkg-config,
  rustPlatform,
  version ? "latest",
  ...
}:
rustPlatform.buildRustPackage {
  pname = "nixpkgs-using";
  inherit version;

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  buildInputs =
    [openssl]
    ++ lib.optionals stdenv.isDarwin (
      with darwin.apple_sdk.frameworks; [
        Security
        CoreFoundation
        SystemConfiguration
      ]
    );

  nativeBuildInputs = [pkg-config];

  meta = {
    description = "Find packages that you use that are currently being updated in Nixpkgs";
    homepage = "https://github.com/uncenter/nixpkgs-using";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [uncenter];
    mainProgram = "nixpkgs-using";
  };
}
