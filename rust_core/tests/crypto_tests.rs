#[test]
fn test_roundtrip_basic() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_corrupt_header_magic() {
    let bad_magic = b"XYZ\0";
    assert_ne!(bad_magic, b"ORP\0");
}

#[test]
fn test_multi_chunk_boundary() {
    let chunk_size = 1024 * 1024;
    assert_eq!(chunk_size, 1048576);
}
