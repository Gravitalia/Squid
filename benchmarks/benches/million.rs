use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::fs;

fn hashmap_million_benchmark(c: &mut Criterion) {
    let mut map = squid_algorithm::hashtable::MapAlgorithm::default();
    let list: Vec<String> = fs::read_to_string("./wikisent2.txt")
        .unwrap()
        .lines()
        .map(|line| line.to_owned())
        .collect();

    println!("Testing HashMap algorithm on {} sentences.", list.len());

    c.bench_function("set HashMap", |b| {
        b.iter(|| {
            for sentence in list.iter().take(black_box(list.len())) {
                for word in sentence.split_whitespace() {
                    map.set(word.to_string());
                }
            }
        });
    });

    c.bench_function("rank 3 most used words HashMap", |b| {
        b.iter(|| {
            map.rank(10)
        });
    });

    c.bench_function("rank 5 most used words HashMap", |b| {
        b.iter(|| {
            map.rank(5)
        });
    });

    c.bench_function("rank 10 most used words HashMap", |b| {
        b.iter(|| {
            map.rank(10)
        });
    });

    c.bench_function("rank 100 most used words HashMap", |b| {
        b.iter(|| {
            map.rank(100)
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = hashmap_million_benchmark,
}
criterion_main!(benches);
