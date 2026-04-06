use crate::error::{ClawError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct SiftDataLoader;

impl SiftDataLoader {
    pub fn load_fvecs<P: AsRef<Path>>(path: P) -> Result<(usize, Vec<Vec<f32>>)> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut vectors = Vec::new();
        let mut dimension = 0;

        while let Ok(d) = reader.read_i32::<LittleEndian>() {
            let dim = d as usize;

            if dimension == 0 {
                dimension = dim;
            } else if dimension != dim {
                return Err(ClawError::LoaderError(format!(
                    "Inconsistent dimensions: expected {}, got {}",
                    dimension, dim
                )));
            }

            let mut vector = vec![0.0f32; dimension];
            for val in vector.iter_mut().take(dimension) {
                *val = reader.read_f32::<LittleEndian>()?;
            }

            vectors.push(vector);
        }

        Ok((dimension, vectors))
    }

    pub fn load_bvecs<P: AsRef<Path>>(path: P) -> Result<(usize, Vec<Vec<f32>>)> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut vectors = Vec::new();
        let mut dimension = 0;

        while let Ok(d) = reader.read_i32::<LittleEndian>() {
            let dim = d as usize;

            if dimension == 0 {
                dimension = dim;
            } else if dimension != dim {
                return Err(ClawError::LoaderError(format!(
                    "Inconsistent dimensions: expected {}, got {}",
                    dimension, dim
                )));
            }

            let mut buffer = vec![0u8; dimension];
            reader.read_exact(&mut buffer)?;

            let vector: Vec<f32> = buffer.iter().map(|&b| b as f32).collect();
            vectors.push(vector);
        }

        Ok((dimension, vectors))
    }

    pub fn load_ivecs<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<i32>>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut vectors = Vec::new();

        while let Ok(d) = reader.read_i32::<LittleEndian>() {
            let dim = d as usize;

            let mut vector = vec![0i32; dim];
            for val in vector.iter_mut().take(dim) {
                *val = reader.read_i32::<LittleEndian>()?;
            }

            vectors.push(vector);
        }

        Ok(vectors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_fvecs_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        let vectors = vec![vec![1.0f32, 2.0, 3.0], vec![4.0f32, 5.0, 6.0]];

        for vec in vectors {
            let dim = vec.len() as i32;
            file.write_all(&dim.to_le_bytes()).unwrap();
            for val in vec {
                file.write_all(&val.to_le_bytes()).unwrap();
            }
        }

        file
    }

    #[test]
    fn test_load_fvecs() {
        let file = create_test_fvecs_file();
        let (dim, vectors) = SiftDataLoader::load_fvecs(file.path()).unwrap();

        assert_eq!(dim, 3);
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(vectors[1], vec![4.0, 5.0, 6.0]);
    }

    fn create_test_bvecs_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        let vectors = vec![vec![10u8, 20, 30], vec![40u8, 50, 60]];

        for vec in vectors {
            let dim = vec.len() as i32;
            file.write_all(&dim.to_le_bytes()).unwrap();
            file.write_all(&vec).unwrap();
        }

        file
    }

    #[test]
    fn test_load_bvecs() {
        let file = create_test_bvecs_file();
        let (dim, vectors) = SiftDataLoader::load_bvecs(file.path()).unwrap();

        assert_eq!(dim, 3);
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors[0], vec![10.0, 20.0, 30.0]);
        assert_eq!(vectors[1], vec![40.0, 50.0, 60.0]);
    }

    fn create_test_ivecs_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        let vectors = vec![vec![1i32, 2, 3], vec![4i32, 5, 6]];

        for vec in vectors {
            let dim = vec.len() as i32;
            file.write_all(&dim.to_le_bytes()).unwrap();
            for val in vec {
                file.write_all(&val.to_le_bytes()).unwrap();
            }
        }

        file
    }

    #[test]
    fn test_load_ivecs() {
        let file = create_test_ivecs_file();
        let vectors = SiftDataLoader::load_ivecs(file.path()).unwrap();

        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors[0], vec![1, 2, 3]);
        assert_eq!(vectors[1], vec![4, 5, 6]);
    }
}
