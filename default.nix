{ pkgs ? import <nixpkgs> {} }:

let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  rust = pkgs.rustPlatform;
in
rust.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}
