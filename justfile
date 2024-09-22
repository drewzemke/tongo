run: 
    TONGO_LOGLEVEL=info,debug cargo run -- --last

build:
    cargo build

test:
    cargo test

install:
    cargo install --path .

start-mongo:
    docker run -p 27017:27017 -d mongo:7

logs:
    #!/usr/bin/env sh
    if [ -d ~/Library ]; then
        log_path="$HOME/Library/Application Support/tongo/tongo.log"
    else
        log_path="$HOME/.local/share/tongo/tongo.log"
    fi
    tail -f "$log_path"
