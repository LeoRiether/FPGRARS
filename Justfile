run +ARGS: 
    cargo run --release -- {{ARGS}}

unb +ARGS:
    cargo run --features "8-bit-display" --release  -- {{ARGS}}
