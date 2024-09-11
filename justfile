run: 
    cargo run -- --last

build:
    cargo build

test:
    cargo test

install:
    cargo install --path .

start-mongo:
    docker run -p 27017:27017 -d mongo:7

