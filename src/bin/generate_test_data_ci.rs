use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn write_fvecs(path: &Path, vectors: &[Vec<f32>]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for vec in vectors {
        let dim = vec.len() as i32;
        file.write_all(&dim.to_le_bytes())?;
        for val in vec {
            file.write_all(&val.to_le_bytes())?;
        }
    }
    Ok(())
}

fn write_ivecs(path: &Path, vectors: &[Vec<i32>]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for vec in vectors {
        let dim = vec.len() as i32;
        file.write_all(&dim.to_le_bytes())?;
        for val in vec {
            file.write_all(&val.to_le_bytes())?;
        }
    }
    Ok(())
}

fn main() {
    let mut args = std::env::args();
    let num_base = args.nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);
    let num_query = 1000;
    let dim = 128;
    let k = 100;

    println!("=== Generating Synthetic SIFT-like Test Data for CI ===");
    println!("Base vectors: {}", num_base);
    println!("Query vectors: {}", num_query);
    println!("Dimensions: {}", dim);
    println!();

    let data_dir = std::path::PathBuf::from("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let mut rng = rand::thread_rng();

    println!("[1/3] Generating base vectors...");
    let base_vectors: Vec<Vec<f32>> = (0..num_base)
        .map(|_| {
            (0..dim)
                .map(|_| rng.gen::<f32>() * 100.0)
                .collect()
        })
        .collect();

    println!("[2/3] Generating query vectors...");
    let query_vectors: Vec<Vec<f32>> = (0..num_query)
        .map(|_| {
            (0..dim)
                .map(|_| rng.gen::<f32>() * 100.0)
                .collect()
        })
        .collect();

    println!("[3/3] Computing ground truth (brute force)...");
    let ground_truth: Vec<Vec<i32>> = query_vectors
        .iter()
        .enumerate()
        .map(|(i, query)| {
            if i % 100 == 0 {
                println!("  Processing query {} / {}", i, num_query);
            }
            let mut distances: Vec<(usize, f64)> = base_vectors
                .iter()
                .enumerate()
                .map(|(j, vec)| {
                    let dist: f64 = vec
                        .iter()
                        .zip(query.iter())
                        .map(|(a, b)| (a - b).powi(2) as f64)
                        .sum();
                    (j, dist.sqrt())
                })
                .collect();
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            distances
                .iter()
                .take(k)
                .map(|(idx, _)| *idx as i32)
                .collect()
        })
        .collect();

    println!();
    println!("Writing files...");
    let base_path = data_dir.join("sift_base.fvecs");
    let query_path = data_dir.join("sift_query.fvecs");
    let gt_path = data_dir.join("sift_groundtruth.ivecs");

    write_fvecs(&base_path, &base_vectors).expect("Failed to write base vectors");
    write_fvecs(&query_path, &query_vectors).expect("Failed to write query vectors");
    write_ivecs(&gt_path, &ground_truth).expect("Failed to write ground truth");

    println!();
    println!("=== Generation Complete ===");
    println!("Files created:");
    println!("  - {}", base_path.display());
    println!("  - {}", query_path.display());
    println!("  - {}", gt_path.display());
}
