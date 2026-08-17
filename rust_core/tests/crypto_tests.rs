use base64::Engine;
use rust_core::crypto;
use rust_core::error::CryptError;
use rust_core::validate;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn tmp_path(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "opencrypt_test_{}_{}_{}",
        std::process::id(),
        n,
        name
    ))
}

fn cleanup(paths: &[&PathBuf]) {
    for p in paths {
        let _ = fs::remove_file(p);
    }
}

fn make_file(path: &PathBuf, size: usize) {
    let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    fs::write(path, data).unwrap();
}

#[test]
fn roundtrip_password_small_file() {
    let input = tmp_path("in.txt");
    let enc = tmp_path("enc.ocrypt");
    let dec = tmp_path("dec.txt");
    make_file(&input, 100_000);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "secret").unwrap();
    crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "secret").unwrap();
    assert_eq!(fs::read(&input).unwrap(), fs::read(&dec).unwrap());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn roundtrip_password_empty_file() {
    let input = tmp_path("empty.txt");
    let enc = tmp_path("empty.ocrypt");
    let dec = tmp_path("empty_out.txt");
    fs::write(&input, b"").unwrap();
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "p").unwrap();
    assert_eq!(fs::read(&input).unwrap(), fs::read(&dec).unwrap());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn empty_file_wrong_password_rejected() {
    let input = tmp_path("empty2.txt");
    let enc = tmp_path("empty2.ocrypt");
    let dec = tmp_path("empty2_out.txt");
    fs::write(&input, b"").unwrap();
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "right").unwrap();
    let e = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "wrong");
    assert!(e.is_err());
    assert!(!dec.exists());
    let v = crypto::verify_password(enc.to_str().unwrap(), "wrong").unwrap();
    assert!(!v);
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn roundtrip_password_large_multichunk_file() {
    let input = tmp_path("big.bin");
    let enc = tmp_path("big.ocrypt");
    let dec = tmp_path("big_out.bin");
    make_file(&input, 3 * 1024 * 1024 + 17);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "s3cr3t!").unwrap();
    crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "s3cr3t!").unwrap();
    assert_eq!(fs::read(&input).unwrap(), fs::read(&dec).unwrap());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn roundtrip_with_key() {
    let input = tmp_path("key_in.txt");
    let enc = tmp_path("key.ocrypt");
    let dec = tmp_path("key_out.txt");
    make_file(&input, 64 * 1024);
    let key = crypto::generate_random_key();
    crypto::encrypt_file_with_key(input.to_str().unwrap(), enc.to_str().unwrap(), &key).unwrap();
    crypto::decrypt_file_with_key(enc.to_str().unwrap(), dec.to_str().unwrap(), &key).unwrap();
    assert_eq!(fs::read(&input).unwrap(), fs::read(&dec).unwrap());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn wrong_password_rejected() {
    let input = tmp_path("wp.txt");
    let enc = tmp_path("wp.ocrypt");
    let dec = tmp_path("wp_out.txt");
    make_file(&input, 5000);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "right").unwrap();
    let r = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "wrong");
    assert!(r.is_err());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn wrong_key_rejected() {
    let input = tmp_path("wk.txt");
    let enc = tmp_path("wk.ocrypt");
    let dec = tmp_path("wk_out.txt");
    make_file(&input, 5000);
    crypto::encrypt_file_with_key(
        input.to_str().unwrap(),
        enc.to_str().unwrap(),
        &crypto::generate_random_key(),
    )
    .unwrap();
    let r = crypto::decrypt_file_with_key(
        enc.to_str().unwrap(),
        dec.to_str().unwrap(),
        &crypto::generate_random_key(),
    );
    assert!(r.is_err());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn tampered_ciphertext_detected() {
    let input = tmp_path("tamper.txt");
    let enc = tmp_path("tamper.ocrypt");
    let dec = tmp_path("tamper_out.txt");
    make_file(&input, 100_000);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    let mut data = fs::read(&enc).unwrap();
    let mid = data.len() / 2;
    data[mid] ^= 0xFF;
    fs::write(&enc, &data).unwrap();
    let r = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "p");
    assert!(r.is_err());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn tampered_header_detected() {
    let input = tmp_path("th.txt");
    let enc = tmp_path("th.ocrypt");
    let dec = tmp_path("th_out.txt");
    make_file(&input, 4096);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    let mut data = fs::read(&enc).unwrap();
    data[4 + 16 + 5] ^= 0xFF;
    fs::write(&enc, &data).unwrap();
    let r = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "p");
    assert!(r.is_err());
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn bad_magic_rejected() {
    let input = tmp_path("bm.txt");
    let enc = tmp_path("bm.ocrypt");
    let dec = tmp_path("bm_out.txt");
    make_file(&input, 100);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    let mut data = fs::read(&enc).unwrap();
    data[0] = b'X';
    fs::write(&enc, &data).unwrap();
    let r = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "p");
    assert!(matches!(r, Err(CryptError::InvalidFormat)));
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn bad_version_rejected() {
    let input = tmp_path("bv.txt");
    let enc = tmp_path("bv.ocrypt");
    let dec = tmp_path("bv_out.txt");
    make_file(&input, 100);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    let mut data = fs::read(&enc).unwrap();
    data[4] = 99;
    fs::write(&enc, &data).unwrap();
    let r = crypto::decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "p");
    assert!(matches!(r, Err(CryptError::InvalidFormat)));
    cleanup(&[&input, &enc, &dec]);
}

#[test]
fn header_has_expected_layout() {
    let input = tmp_path("fmt.txt");
    let enc = tmp_path("fmt.ocrypt");
    make_file(&input, 2048);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "p").unwrap();
    let data = fs::read(&enc).unwrap();
    assert_eq!(&data[0..4], b"ORP\0");
    assert_eq!(data[4], 1);
    assert_eq!(data.len(), 41 + 2048 + 16);
    cleanup(&[&input, &enc]);
}

#[test]
fn same_file_same_password_produces_different_output() {
    let input = tmp_path("uniq.txt");
    let enc1 = tmp_path("uniq1.ocrypt");
    let enc2 = tmp_path("uniq2.ocrypt");
    make_file(&input, 4096);
    crypto::encrypt_file(input.to_str().unwrap(), enc1.to_str().unwrap(), "p").unwrap();
    crypto::encrypt_file(input.to_str().unwrap(), enc2.to_str().unwrap(), "p").unwrap();
    let a = fs::read(&enc1).unwrap();
    let b = fs::read(&enc2).unwrap();
    assert_ne!(a, b);
    cleanup(&[&input, &enc1, &enc2]);
}

#[test]
fn verify_password_ok_and_fail() {
    let input = tmp_path("vp.txt");
    let enc = tmp_path("vp.ocrypt");
    make_file(&input, 4096);
    crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pass").unwrap();
    assert!(crypto::verify_password(enc.to_str().unwrap(), "pass").unwrap());
    assert!(!crypto::verify_password(enc.to_str().unwrap(), "nope").unwrap());
    cleanup(&[&input, &enc]);
}

#[test]
fn random_key_is_urlsafe_base64_32_bytes() {
    let key = crypto::generate_random_key();
    assert_eq!(key.len(), 43);
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&key)
        .unwrap();
    assert_eq!(decoded.len(), 32);
}

#[test]
fn keys_are_unique() {
    let a = crypto::generate_random_key();
    let b = crypto::generate_random_key();
    assert_ne!(a, b);
}

#[test]
fn secure_delete_removes_file() {
    let path = tmp_path("del.bin");
    fs::write(&path, vec![0xAB; 8192]).unwrap();
    crypto::secure_delete(path.to_str().unwrap()).unwrap();
    assert!(!path.exists());
}

#[test]
fn validate_rejects_missing_input() {
    let input = tmp_path("missing.bin");
    let out = tmp_path("out.ocrypt");
    let r = validate::validate_paths(input.to_str().unwrap(), out.to_str().unwrap(), false);
    assert!(matches!(r, Err(CryptError::InputInvalid)));
}

#[test]
fn validate_rejects_existing_output() {
    let input = tmp_path("vin.txt");
    let out = tmp_path("vout.ocrypt");
    fs::write(&input, b"x").unwrap();
    fs::write(&out, b"y").unwrap();
    let r = validate::validate_paths(input.to_str().unwrap(), out.to_str().unwrap(), false);
    assert!(matches!(r, Err(CryptError::OutputExists)));
    cleanup(&[&input, &out]);
}

#[test]
fn validate_rejects_same_path() {
    let input = tmp_path("vsame.txt");
    fs::write(&input, b"x").unwrap();
    let r = validate::validate_paths(input.to_str().unwrap(), input.to_str().unwrap(), false);
    assert!(matches!(r, Err(CryptError::InputInvalid)));
    cleanup(&[&input]);
}

#[test]
fn validate_rejects_wrong_extension_for_decrypt() {
    let input = tmp_path("vdec.txt");
    let out = tmp_path("vdec_out.txt");
    fs::write(&input, b"x").unwrap();
    let r = validate::validate_paths(input.to_str().unwrap(), out.to_str().unwrap(), true);
    assert!(matches!(r, Err(CryptError::InputInvalid)));
    cleanup(&[&input, &out]);
}

#[test]
fn encrypt_rejects_existing_output_and_preserves_it() {
    let input = tmp_path("oe_in.txt");
    let enc = tmp_path("oe.ocrypt");
    fs::write(&input, b"x").unwrap();
    fs::write(&enc, b"y").unwrap();
    let r = crypto::encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw");
    assert!(matches!(r, Err(CryptError::OutputExists)));
    assert_eq!(fs::read(&enc).unwrap(), b"y");
    cleanup(&[&input, &enc]);
}

#[test]
fn encrypt_rejects_same_path() {
    let input = tmp_path("sp.txt");
    fs::write(&input, b"x").unwrap();
    let r = crypto::encrypt_file(input.to_str().unwrap(), input.to_str().unwrap(), "pw");
    assert!(matches!(r, Err(CryptError::InputInvalid)));
    cleanup(&[&input]);
}
