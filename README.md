# `margit` -- a TUI for MongoDB

*(A) Mongo App (that is) Really Great In (your) Terminal!*

## Installation

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then run:
```shell
cargo install mongodb-tui
```

## Usage

For now, you have to pass the connection string along with the database and collection you want to view.
Future versions will allow choosing the database and collection from within the app.

```shell
margit --url mongodb://localhost:27017 --database database_name --collection collection_name
```

