run +ARGS: 
    cargo run --release -- {{ARGS}}

build-unb:
    cargo build --features unb --release

unb +ARGS:
    cargo run --features unb --release  -- {{ARGS}}
