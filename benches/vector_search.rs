use clawdb::{DistanceMetric, Vector, VectorStorage};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;

fn bench_insert(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let storage = VectorStorage::open(temp_dir.path(), 128, DistanceMetric::Euclidean).unwrap();

    let mut group = c.benchmark_group("insert");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let vectors: Vec<Vector> = (0..size)
                .map(|i| Vector::new(i as u64, vec![1.0; 128]))
                .collect();

            b.iter(|| {
                for v in vectors.iter() {
                    storage.insert(v.clone()).unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut storage = VectorStorage::open(temp_dir.path(), 128, DistanceMetric::Euclidean).unwrap();

    for i in 0..10000 {
        let vector = Vector::new(i as u64, vec![i as f32 / 10000.0; 128]);
        storage.insert(vector).unwrap();
    }

    storage.build_index(100).unwrap();

    let query = vec![0.5; 128];

    c.bench_function("search_k_10", |b| {
        b.iter(|| storage.search(black_box(&query), 10, 10).unwrap())
    });

    c.bench_function("brute_force_search_k_10", |b| {
        b.iter(|| storage.brute_force_search(black_box(&query), 10).unwrap())
    });
}

criterion_group!(benches, bench_insert, bench_search);
criterion_main!(benches);
