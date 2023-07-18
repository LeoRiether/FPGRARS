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

bench:
    hyperfine --warmup 5 \
        './fpgrars-big-match --no-video samples/bench/add.s' \
        './fpgrars-closures-cps --no-video samples/bench/add.s' \
        './fpgrars-closures-cps-nbc --no-video samples/bench/add.s'

bench-mem:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video samples/bench/memory.s' \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video samples/bench/memory2.s' \

bench-spike-add:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video samples/bench/add.s' \
        --prepare 'cd ../spike/ && just build add' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/add'

bench-spike-mem:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video samples/bench/memory.s' \
        --prepare 'cd ../spike/ && just build memory' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/memory'

bench-spike-sort:
    hyperfine --warmup 3 \
        --prepare 'just build-unb' \
        './target/release/fpgrars --no-video samples/bench/sort.s' \
        --prepare '' \
        '/home/leonardo/Desktop/bench/fpgrars-v2 --no-video samples/bench/sort.s' \
        --prepare 'cd ../spike/ && just build sort' \
        'spike --isa=RV32IMAC /home/leonardo/Workspace/TG/RISCV/riscv32-unknown-elf/bin/pk ../spike/sort'
