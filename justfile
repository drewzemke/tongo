run: 
    TONGO_LOGLEVEL=info,debug cargo run -- --last

build:
    cargo build

test:
    cargo test

install:
    cargo install --path .

start-mongo:
    docker compose up -d \
    && sleep 2 \
    && ./scripts/seed.sh

logs:
    #!/usr/bin/env sh
    if [ -d ~/Library ]; then
        log_path="$HOME/Library/Application Support/tongo/tongo.log"
    else
        log_path="$HOME/.local/share/tongo/tongo.log"
    fi
    tail -f "$log_path"
