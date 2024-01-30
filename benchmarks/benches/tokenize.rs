use criterion::{criterion_group, criterion_main, Criterion};

fn tokenize_benchmark(c: &mut Criterion) {
    c.bench_function("tokenize", |b| {
        // To do.
        b.iter(|| println!("tokenize"))
    });
}

criterion_group!(benches, tokenize_benchmark);
criterion_main!(benches);
