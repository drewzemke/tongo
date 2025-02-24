# `tongo` -- a TUI for MongoDB

`tongo` is a fast and keyboard-driven TUI (Terminal User Interface) for MongoDB.
Navigate and manipulate your data without leaving your terminal.

## Major Features

- 🔌 Connect & save MongoDB connections 
- ⚡️ Quickly navigate your data with customizable keybindings
- 📝 Create and edit documents using your terminal editor of choice

## Installation

### Using `cargo`

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then install:
```shell
$ cargo install tongo
```

### Using `nix`

Install [Nix](https://nixos.org/), then clone this repo and install:
```shell
$ git clone git@github.com:drewzemke/tongo.git
$ cd tongo
$ nix-build
```

Then you can move the created binary somewhere on your path:
```shell
$ cp ./result/bin/tongo /usr/local/bin/tongo
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

Restore your most-recently-closed session in the app.
```shell
$ tongo --last
```

