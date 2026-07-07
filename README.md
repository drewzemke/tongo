# `tongo` -- a TUI for MongoDB

`tongo` is a fast and keyboard-driven TUI (Terminal User Interface) for MongoDB.
Navigate and manipulate your data without leaving your terminal.

![Demo](./assets/recordings/demo.gif)

## Major Features

- Connect & save MongoDB connections 
- Quickly navigate your data with customizable keybindings
- Filter your data using Mongo queries
- Create and edit documents using your terminal editor of choice
- Fuzzy search currently-visible data to quickly drill down to what you're looking for
- Run multiple sessions in tabs for quick data comparisons between collections
- Copy data directly to the system clipboard
- Browse your data in style with customizable color themes

## Installation

### Using `cargo`

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then install:
```shell
cargo install tongo
```

### Using `nix`

Install [Nix](https://nixos.org/), then clone this repo and install:
```shell
git clone git@github.com:drewzemke/tongo.git
cd tongo
nix-build
```

Then you can move the created binary somewhere on your path:
```shell
cp ./result/bin/tongo /usr/local/bin/tongo
```

### Using AUR

If you're on Arch Linux, you can install `tongo` [from the AUR](https://aur.archlinux.org/packages/tongo):
```shell
paru -S tongo
```

### Using Homebrew

If you're on macOS, install [Homebrew](https://brew.sh/) and then run
```shell
brew install drewzemke/tap/tongo
```

## Quick Start

1. Launch `tongo` in your terminal of choice:
```shell
tongo
```
2. Start creating a new connection by pressing `A`
3. Set a name for the connection and enter your database's connection string
4. After connecting, select a database and then a connection to connect to
5. Use the arrow keys to navigate through the data. Press `n` and `p` to move between pages

At any time (except when inputting text), you can bring up a commands list by pressing `?` that will explain what actions are available to you and what their keybindings are.


## Usage

Load a connection directly:
```shell
tongo --url mongodb://localhost:27017 
```

If you've previously stored a connection, you can load it by name:
```shell
tongo --connection local
```

Restore your most-recently-closed session in the app:
```shell
tongo --last
```


## Configuration

The first time you run `tongo`, a `config.toml` will be created for you in `~/.config/tongo` on Mac and Linux and in `<your-user-folder>\AppData\Roaming\tongo` on Windows. (You can also see that file [here](./assets/default-config.toml).) It contains a full commented-out configuration together with descriptions of each configuration point. 

### Color Themes

`tongo` detects your terminal's background at startup and picks light- or dark-friendly default colors so the UI stays legible either way.

You can fully customize the colors used in `tongo`'s UI to your liking by creating a `theme.toml` in the same directory as your configuration file; any colors you set there take precedence over the detected defaults. Check out [our small themes collection](./assets/themes) for some examples to get you started.


## Contributing

Please open an issue if you run into a problem while using `tongo`, or if there's a piece of functionality you wish it had! You're also welcome to make changes yourself and open a PR.
