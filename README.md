# `tongo` -- a TUI for MongoDB

## Installation

### Using `cargo`

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then install:
```shell
$ cargo install tongo
```

### Using `nix`

Install [Nix](https://nixos.org/), then install:
```shell
$ nix-shell -p cargo # or clone the tongo repo and run `nix-shell` from there
$ cargo install tongo
```

## Usage

```shell
$ tongo 
```

Load a connection directly:
```shell
$ tongo --url mongodb://localhost:27017 
```

If you've previously stored a connection, you can load it by name:
```shell
$ tongo --connection local
```

