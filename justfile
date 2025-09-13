run: 
    TONGO_LOGLEVEL=info,debug cargo run -- --last

build:
    cargo build

test:
    cargo test

watch:
    watchexec --wrap-process=session -r just run 

install:
    cargo install --path .

start-mongo:
    docker compose up -d \
    && sleep 2 \
    && ./scripts/seed.sh

logs:
    tail -f $HOME/.local/share/tongo/tongo.log

record-demo:
    vhs assets/demo.tape
