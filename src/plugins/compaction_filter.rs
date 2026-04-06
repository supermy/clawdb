const TOMBSTONE_MARKER: &[u8] = b"TOMBSTONE";

pub fn create_vector_compaction_filter() -> impl Fn(u32, &[u8], &[u8]) -> Option<Vec<u8>> {
    move |_level: u32, _key: &[u8], value: &[u8]| {
        if value.starts_with(TOMBSTONE_MARKER) {
            None
        } else {
            Some(value.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tombstone_detection() {
        let filter = create_vector_compaction_filter();

        let tombstone_value = b"TOMBSTONE".to_vec();
        let result = filter(0, b"key1", &tombstone_value);
        assert!(result.is_none());

        let normal_value = b"normal_vector_data".to_vec();
        let result = filter(0, b"key2", &normal_value);
        assert!(result.is_some());
    }
}
