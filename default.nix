{
  lib,
  rustPlatform,
  version ? "latest",
  ...
}:
rustPlatform.buildRustPackage {
  pname = "nixpkgs-using";
  inherit version;

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "Find packages that you use that are currently being updated in Nixpkgs.";
    homepage = "https://github.com/uncenter/nixpkgs-using";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [uncenter];
    mainProgram = "nixpkgs-using";
  };
}
