run +ARGS: 
    cargo run --release -- {{ARGS}}

build-unb:
    cargo build --features "8-bit-display" --release

unb +ARGS:
    cargo run --features "8-bit-display" --release  -- {{ARGS}}
