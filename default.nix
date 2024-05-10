{
  lib,
  rustPlatform,
  version ? "latest",
  pkg-config,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "nixpkgs-using";
  inherit version;

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    pkg-config
  ];

  meta = {
    description = "Find packages that you use that are currently being updated in Nixpkgs.";
    homepage = "https://github.com/uncenter/nixpkgs-using";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [uncenter];
    mainProgram = "nixpkgs-using";
  };
}
