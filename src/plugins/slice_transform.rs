pub fn extract_collection_prefix(key: &[u8]) -> Option<&[u8]> {
    if key.len() >= 4 {
        Some(&key[..4])
    } else {
        None
    }
}

pub fn is_in_collection_domain(key: &[u8]) -> bool {
    key.len() >= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_extraction() {
        let key = b"\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x01";
        let prefix = extract_collection_prefix(key);
        assert_eq!(prefix, Some(b"\x00\x00\x00\x01".as_ref()));

        assert!(is_in_collection_domain(key));
        assert!(!is_in_collection_domain(b"abc"));
    }
}
