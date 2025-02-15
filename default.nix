{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/0ff09db9d034a04acd4e8908820ba0b410d7a33a.tar.gz") {} }:

pkgs.mkShell {
  packages = with pkgs; [
    pkgs.cargo
    pkgs.git
    pkgs.just
  ];
}

