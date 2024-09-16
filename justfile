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
    if [ -n "$XDG_DATA_HOME" ]; then
        log_path="$XDG_DATA_HOME/tongo/tongo.log"
    else
        log_path="$HOME/.local/share/tongo/tongo.log"
    fi
    tail -f $log_path
