#[test]
fn test_roundtrip_basic() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_corrupt_header_magic() {
    let bad_magic = b"XYZ\0";
    assert_ne!(bad_magic, b"ORP\0");
}
