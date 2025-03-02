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
    tail -f $HOME/.local/share/tongo/tongo.log
