use criterion::{criterion_group, criterion_main, Criterion};
use squid::helpers::tokenizer::tokenize;

fn tokenize_benchmark(c: &mut Criterion) {
    const FRENCH: &str =
        "Le soleil brille, illuminant la ville endormie. Les rues sont calmes, baignées dans une douce lumière. Au loin, les oiseaux oisifs chantent la vie !";

    c.bench_function("tokenize 150 bytes", |b| b.iter(|| tokenize(FRENCH)));
}

criterion_group!(benches, tokenize_benchmark);
criterion_main!(benches);
