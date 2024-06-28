{ pkgs ? import <nixpkgs> {} }:

let
  rust = pkgs.rustPlatform;
in
rust.buildRustPackage rec {
  pname = "mongodb-tui";
  version = "0.3.1"; 

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}
