{ pkgs ? import <nixpkgs> {} }:

let
  rust = pkgs.rustPlatform;
in
rust.buildRustPackage rec {
  pname = "mongodb-tui";
  version = "0.5.0"; 

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}
