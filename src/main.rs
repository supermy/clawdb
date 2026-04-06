use clawdb::{DistanceMetric, Vector, VectorStorage};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== ClawDB Vector Database Demo ===\n");

    let db_path = "./vector_db";
    let dimension = 128;

    println!("Opening vector database at {}...", db_path);
    let mut storage = VectorStorage::open(db_path, dimension, DistanceMetric::Euclidean)?;

    let demo_vectors = create_demo_vectors(1000, dimension);
    println!("Inserting {} demo vectors...", demo_vectors.len());

    let start = Instant::now();
    storage.insert_batch(demo_vectors)?;
    let insert_time = start.elapsed();
    println!("Insert completed in {:?}", insert_time);

    let count = storage.count()?;
    println!("Total vectors in database: {}\n", count);

    println!("Building IVF index with 100 clusters...");
    let start = Instant::now();
    storage.build_index(100)?;
    let build_time = start.elapsed();
    println!("Index built in {:?}\n", build_time);

    let query = vec![0.5; dimension];
    let k = 10;

    println!("Searching for {} nearest neighbors using index...", k);
    let start = Instant::now();
    let results = storage.search(&query, k, 10)?;
    let search_time = start.elapsed();

    println!("Index search completed in {:?}", search_time);
    println!("Top {} results:", results.len());
    for (i, (id, distance)) in results.iter().enumerate() {
        println!("  {}. ID: {}, Distance: {:.4}", i + 1, id, distance);
    }

    println!("\nSearching with brute force...");
    let start = Instant::now();
    let brute_results = storage.brute_force_search(&query, k)?;
    let brute_time = start.elapsed();

    println!("Brute force search completed in {:?}", brute_time);
    println!("Top {} results:", brute_results.len());
    for (i, (id, distance)) in brute_results.iter().enumerate() {
        println!("  {}. ID: {}, Distance: {:.4}", i + 1, id, distance);
    }

    let speedup = brute_time.as_secs_f64() / search_time.as_secs_f64();
    println!("\nIndex search is {:.2}x faster than brute force", speedup);

    storage.flush()?;
    storage.compact()?;

    println!("\nDatabase flushed and compacted successfully!");

    Ok(())
}

fn create_demo_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..dimension)
                .map(|j| (i as f32 + j as f32 * 0.1) % 1.0)
                .collect();
            Vector::new(i as u64, data)
        })
        .collect()
}
