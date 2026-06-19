{
  pname,
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  inherit pname;
  inherit ((fromTOML (builtins.readFile ./Cargo.toml)).package) version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
