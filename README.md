# `tongo` -- a TUI for MongoDB

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

If you've previously store a connection, you can load it by name:
```shell
$ tongo --connection local
```

