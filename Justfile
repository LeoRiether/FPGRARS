version := `cargo metadata --format-version=1 --no-deps | yq '.packages[].version'`

run +ARGS: 
    cargo run --release -- {{ARGS}}

build-unb:
    cargo build --features unb --release

unb +ARGS:
    cargo run --features unb --release  -- {{ARGS}}

publish version:
    cargo test
    git tag {{version}}
    git push --tags
    cargo publish
    mkdocs gh-deploy
    printf "\x1b[96mFPGRARS {{version}} is published! Please manually create a new release in GitHub with the tag {{version}}\x1b[0m\n"

build-copy:
    cargo build --release
    cp ./target/release/fpgrars ./fpgrars-v{{version}}

bench case v1 v2 *args:
    hyperfine --warmup 5 {{args}} \
        './{{v1}} --no-video benches/samples/{{case}}.s' \
        './{{v2}} --no-video benches/samples/{{case}}.s' \

bench-3 case v1 v2 v3 *args:
    hyperfine --warmup 5 {{args}} \
        './{{v1}} --no-video benches/samples/{{case}}.s' \
        './{{v2}} --no-video benches/samples/{{case}}.s' \
        './{{v3}} --no-video benches/samples/{{case}}.s' \

_bench-deprecated:
    hyperfine --warmup 5 \
        './fpgrars-big-match --no-video benches/samples/add.s' \
        './fpgrars-closures-cps --no-video benches/samples/add.s' \
        './fpgrars-closures-cps-nbc --no-video benches/samples/add.s'

_bench-mem:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/memory.s' \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/memory2.s' \

_bench-spike-add:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/add.s' \
        --prepare 'cd ../spike/ && just build add' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/add'

_bench-spike-mem:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/memory.s' \
        --prepare 'cd ../spike/ && just build memory' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/memory'

_bench-spike-sort:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/sort.s' \
        --prepare '' \
        '/home/leonardo/Desktop/bench/fpgrars-v2 --no-video benches/samples/sort.s' \
        --prepare 'cd ../spike/ && just build sort' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/sort'

_bench-versions-add:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video benches/samples/add.s' \
        --prepare '' \
        './fpgrars-v2.3 --no-video benches/samples/add.s' \
        --prepare '' \
        './fpgrars-v2.4 --no-video benches/samples/add.s' \

lib := 'fpgrars::simulator::executor'
asm *args:
    cargo asm -p fpgrars --rust --lib {{lib}} {{args}}
