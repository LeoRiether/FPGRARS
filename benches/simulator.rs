use criterion::{criterion_group, criterion_main, Criterion};
use fpgrars::simulator::{self, Simulator};

const TESTCASES: &[&str] = &["add.s", "memory.s", "memory2.s", "sort.s", "video.s"];

fn criterion_benchmark(c: &mut Criterion) {
    for testcase in TESTCASES {
        c.bench_function(testcase, |b| {
            let memory = simulator::memory::Memory::new();
            let mut simulator = Simulator::default().with_memory(memory);
            simulator
                .load_file(&format!("./benches/samples/{testcase}"))
                .unwrap_or_else(|e| panic!("Couldn't parse {testcase}: {e}"));

            b.iter(|| simulator.run())
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
