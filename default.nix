{ pkgs ? import <nixpkgs> {} }:

let
  rust = pkgs.rustPlatform;
in
rust.buildRustPackage rec {
  pname = "tongo";
  version = "0.8.1"; 

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}
