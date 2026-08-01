use crate::error::{CryptError, Result};
use crate::validate;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key};
use argon2::Argon2;
use base64::Engine;
use generic_array::GenericArray;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use zeroize::Zeroize;

pub const MAGIC: &[u8] = b"ORP\0";
pub const FORMAT_VERSION: u8 = 1;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const CHUNK_SIZE: usize = 1024 * 1024;
const TAG_LEN: usize = 16;
const PLAINTEXT_LEN_LEN: usize = 8;

pub const HEADER_LEN: usize = MAGIC.len() + 1 + PLAINTEXT_LEN_LEN + SALT_LEN + NONCE_LEN;

struct Header {
    plaintext_len: u64,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let ctx = Argon2::default();
    ctx.hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptError::Memory)?;
    Ok(key)
}

fn make_nonce(base: &[u8; NONCE_LEN], counter: u32) -> [u8; NONCE_LEN] {
    let mut n = *base;
    n[8] = (counter >> 24) as u8;
    n[9] = (counter >> 16) as u8;
    n[10] = (counter >> 8) as u8;
    n[11] = counter as u8;
    n
}

fn decode_key(b64: &str) -> Result<[u8; 32]> {
    let mut decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(b64)
        .map_err(|_| CryptError::InvalidParam)?;
    if decoded.len() != 32 {
        decoded.zeroize();
        return Err(CryptError::InvalidParam);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    decoded.zeroize();
    Ok(key)
}

fn read_header(file: &mut File) -> Result<Header> {
    let mut magic = [0u8; MAGIC.len()];
    file.read_exact(&mut magic)?;
    if magic != MAGIC {
        return Err(CryptError::InvalidFormat);
    }
    let mut ver = [0u8; 1];
    file.read_exact(&mut ver)?;
    if ver[0] != FORMAT_VERSION {
        return Err(CryptError::InvalidFormat);
    }
    let mut len_bytes = [0u8; PLAINTEXT_LEN_LEN];
    file.read_exact(&mut len_bytes)?;
    let plaintext_len = u64::from_le_bytes(len_bytes);
    let mut salt = [0u8; SALT_LEN];
    file.read_exact(&mut salt)?;
    let mut nonce = [0u8; NONCE_LEN];
    file.read_exact(&mut nonce)?;
    Ok(Header {
        plaintext_len,
        salt,
        nonce,
    })
}

fn write_header(writer: &mut BufWriter<File>, header: &Header) -> Result<()> {
    writer.write_all(MAGIC)?;
    writer.write_all(&[FORMAT_VERSION])?;
    writer.write_all(&header.plaintext_len.to_le_bytes())?;
    writer.write_all(&header.salt)?;
    writer.write_all(&header.nonce)?;
    Ok(())
}

fn read_exact_or_eof(file: &mut File, buf: &mut [u8]) -> Result<usize> {
    let mut offset = 0;
    while offset < buf.len() {
        match file.read(&mut buf[offset..]) {
            Ok(0) => break,
            Ok(n) => offset += n,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(offset)
}

fn encrypt_stream(
    reader: &mut File,
    writer: &mut BufWriter<File>,
    cipher: &Aes256Gcm,
    nonce_arr: &[u8; NONCE_LEN],
    plaintext_len: u64,
) -> Result<()> {
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut counter: u32 = 0;
    let mut written: u64 = 0;
    loop {
        let n = read_exact_or_eof(reader, &mut buf)?;
        if n == 0 && counter > 0 {
            break;
        }
        let nce = make_nonce(nonce_arr, counter);
        let nce_ga = GenericArray::from_slice(&nce);
        let ct = cipher
            .encrypt(nce_ga, &buf[..n])
            .map_err(|_| CryptError::Memory)?;
        writer.write_all(&ct)?;
        written += n as u64;
        counter += 1;
    }
    if written != plaintext_len {
        return Err(CryptError::Io(std::io::Error::other(
            "input size changed during encryption",
        )));
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(())
}

fn decrypt_stream(
    reader: &mut File,
    writer: &mut BufWriter<File>,
    cipher: &Aes256Gcm,
    header: &Header,
) -> Result<()> {
    let mut buf = vec![0u8; CHUNK_SIZE + TAG_LEN];
    let mut counter: u32 = 0;
    let mut written: u64 = 0;
    loop {
        let n = read_exact_or_eof(reader, &mut buf)?;
        if n == 0 {
            break;
        }
        let nce = make_nonce(&header.nonce, counter);
        let nce_ga = GenericArray::from_slice(&nce);
        let pt = cipher
            .decrypt(nce_ga, &buf[..n])
            .map_err(|_| CryptError::InvalidPassword)?;
        written += pt.len() as u64;
        writer.write_all(&pt)?;
        counter += 1;
    }
    if written != header.plaintext_len {
        return Err(CryptError::InvalidFormat);
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(())
}

fn decrypt_with_cleanup(
    reader: &mut File,
    output: &str,
    cipher: &Aes256Gcm,
    header: &Header,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(output)?);
    if let Err(e) = decrypt_stream(reader, &mut writer, cipher, header) {
        let _ = std::fs::remove_file(output);
        return Err(e);
    }
    Ok(())
}

fn check_output(input: &str, output: &str) -> Result<()> {
    let in_path = Path::new(input);
    let out_path = Path::new(output);
    if validate::same_path(in_path, out_path) {
        return Err(CryptError::InputInvalid);
    }
    if out_path.exists() {
        return Err(CryptError::OutputExists);
    }
    Ok(())
}

pub fn encrypt_file(input: &str, output: &str, password: &str) -> Result<()> {
    check_output(input, output)?;
    let plaintext_len = std::fs::metadata(input)?.len();
    let mut reader = File::open(input)?;
    let mut salt = [0u8; SALT_LEN];
    getrandom::getrandom(&mut salt).map_err(|_| CryptError::Memory)?;
    let mut key_bytes = derive_key(password.as_bytes(), &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_raw = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_arr: [u8; NONCE_LEN] = nonce_raw.into();

    let header = Header {
        plaintext_len,
        salt,
        nonce: nonce_arr,
    };
    let r = (|| {
        let mut writer = BufWriter::new(File::create(output)?);
        write_header(&mut writer, &header)?;
        encrypt_stream(&mut reader, &mut writer, &cipher, &nonce_arr, plaintext_len)
    })();
    key_bytes.zeroize();
    if r.is_err() {
        let _ = std::fs::remove_file(output);
    }
    r
}

pub fn decrypt_file(input: &str, output: &str, password: &str) -> Result<()> {
    check_output(input, output)?;
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    let mut key_bytes = derive_key(password.as_bytes(), &header.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let result = decrypt_with_cleanup(&mut reader, output, &cipher, &header);
    key_bytes.zeroize();
    result
}

pub fn encrypt_file_with_key(input: &str, output: &str, b64_key: &str) -> Result<()> {
    check_output(input, output)?;
    let plaintext_len = std::fs::metadata(input)?.len();
    let mut raw_key = decode_key(b64_key)?;
    let cipher = {
        let key = Key::<Aes256Gcm>::from_slice(&raw_key);
        Aes256Gcm::new(key)
    };
    raw_key.zeroize();
    let nonce_raw = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_arr: [u8; NONCE_LEN] = nonce_raw.into();

    let mut salt = [0u8; SALT_LEN];
    getrandom::getrandom(&mut salt).map_err(|_| CryptError::Memory)?;

    let mut reader = File::open(input)?;
    let header = Header {
        plaintext_len,
        salt,
        nonce: nonce_arr,
    };
    let r = (|| {
        let mut writer = BufWriter::new(File::create(output)?);
        write_header(&mut writer, &header)?;
        encrypt_stream(&mut reader, &mut writer, &cipher, &nonce_arr, plaintext_len)
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(output);
    }
    r
}

pub fn decrypt_file_with_key(input: &str, output: &str, b64_key: &str) -> Result<()> {
    check_output(input, output)?;
    let mut raw_key = decode_key(b64_key)?;
    let cipher = {
        let key = Key::<Aes256Gcm>::from_slice(&raw_key);
        Aes256Gcm::new(key)
    };
    raw_key.zeroize();
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    decrypt_with_cleanup(&mut reader, output, &cipher, &header)
}

pub fn verify_password(input: &str, password: &str) -> Result<bool> {
    let mut reader = File::open(input)?;
    let header = read_header(&mut reader)?;
    let mut key_bytes = derive_key(password.as_bytes(), &header.salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut buf = vec![0u8; CHUNK_SIZE + TAG_LEN];
    let n = read_exact_or_eof(&mut reader, &mut buf)?;
    if n == 0 {
        key_bytes.zeroize();
        return Ok(true);
    }
    let nce = make_nonce(&header.nonce, 0);
    let nce_ga = GenericArray::from_slice(&nce);
    let ok = cipher.decrypt(nce_ga, &buf[..n]).is_ok();
    key_bytes.zeroize();
    Ok(ok)
}

pub fn secure_delete(path: &str) -> Result<()> {
    let len = std::fs::metadata(path)?.len();
    if len > 0 {
        let mut f = std::fs::OpenOptions::new().write(true).open(path)?;
        let block = 65536usize;
        let mut buf = vec![0u8; block];
        getrandom::getrandom(&mut buf).map_err(|_| CryptError::Memory)?;
        let mut remaining = len;
        while remaining > 0 {
            let n = remaining.min(block as u64) as usize;
            f.write_all(&buf[..n])?;
            remaining -= n as u64;
        }
        f.flush()?;
        f.sync_all()?;
    }
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn generate_random_key() -> String {
    let mut key = [0u8; 32];
    if getrandom::getrandom(&mut key).is_err() {
        panic!("OS RNG failed");
    }
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("opencrypt_test_{}_{}", std::process::id(), name));
        p
    }

    fn cleanup(paths: &[PathBuf]) {
        for p in paths {
            let _ = std::fs::remove_file(p);
        }
    }

    #[test]
    fn roundtrip_password() {
        let input = tmp("in.txt");
        let enc = tmp("enc.ocrypt");
        let dec = tmp("dec.txt");
        std::fs::write(&input, b"hello world").unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "secret").unwrap();
        decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "secret").unwrap();
        assert_eq!(std::fs::read(&dec).unwrap(), b"hello world");
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn roundtrip_empty_file() {
        let input = tmp("empty_in.txt");
        let enc = tmp("empty.ocrypt");
        let dec = tmp("empty_out.txt");
        std::fs::write(&input, b"").unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();
        decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "pw").unwrap();
        assert_eq!(std::fs::read(&dec).unwrap(), b"");
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn roundtrip_large_file_multiple_chunks() {
        let input = tmp("big_in.bin");
        let enc = tmp("big.ocrypt");
        let dec = tmp("big_out.bin");
        let data: Vec<u8> = (0..(CHUNK_SIZE * 2 + 12345) as u32)
            .map(|i| (i % 251) as u8)
            .collect();
        std::fs::write(&input, &data).unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();
        decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "pw").unwrap();
        assert_eq!(std::fs::read(&dec).unwrap(), data);
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn wrong_password_fails() {
        let input = tmp("wp_in.txt");
        let enc = tmp("wp.ocrypt");
        let dec = tmp("wp_out.txt");
        std::fs::write(&input, b"data").unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "right").unwrap();
        let r = decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "wrong");
        assert!(matches!(r, Err(CryptError::InvalidPassword)));
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn tampered_ciphertext_rejected() {
        let input = tmp("tam_in.txt");
        let enc = tmp("tam.ocrypt");
        let dec = tmp("tam_out.txt");
        std::fs::write(&input, b"tamper me").unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();
        let mut bytes = std::fs::read(&enc).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0x40;
        std::fs::write(&enc, &bytes).unwrap();

        let r = decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "pw");
        assert!(matches!(r, Err(CryptError::InvalidPassword)));
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn truncated_file_rejected() {
        let input = tmp("tr_in.bin");
        let enc = tmp("tr.ocrypt");
        let dec = tmp("tr_out.bin");
        let data: Vec<u8> = (0..(CHUNK_SIZE * 2) as u32)
            .map(|i| (i % 7) as u8)
            .collect();
        std::fs::write(&input, &data).unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();
        let bytes = std::fs::read(&enc).unwrap();
        let cut = bytes.len() - (CHUNK_SIZE + TAG_LEN);
        std::fs::write(&enc, &bytes[..cut]).unwrap();

        let r = decrypt_file(enc.to_str().unwrap(), dec.to_str().unwrap(), "pw");
        assert!(matches!(r, Err(CryptError::InvalidFormat)));
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn key_roundtrip() {
        let input = tmp("key_in.txt");
        let enc = tmp("key.ocrypt");
        let dec = tmp("key_out.txt");
        std::fs::write(&input, b"key mode data").unwrap();
        let key = generate_random_key();

        encrypt_file_with_key(input.to_str().unwrap(), enc.to_str().unwrap(), &key).unwrap();
        decrypt_file_with_key(enc.to_str().unwrap(), dec.to_str().unwrap(), &key).unwrap();
        assert_eq!(std::fs::read(&dec).unwrap(), b"key mode data");
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn bad_key_rejected() {
        let input = tmp("bk_in.txt");
        let enc = tmp("bk.ocrypt");
        let dec = tmp("bk_out.txt");
        std::fs::write(&input, b"x").unwrap();
        let key = generate_random_key();

        encrypt_file_with_key(input.to_str().unwrap(), enc.to_str().unwrap(), &key).unwrap();
        let r = decrypt_file_with_key(
            enc.to_str().unwrap(),
            dec.to_str().unwrap(),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
        assert!(matches!(r, Err(CryptError::InvalidPassword)));
        cleanup(&[input, enc, dec]);
    }

    #[test]
    fn nonce_unique_across_files_same_key() {
        let input1 = tmp("n1_in.txt");
        let input2 = tmp("n2_in.txt");
        let enc1 = tmp("n1.ocrypt");
        let enc2 = tmp("n2.ocrypt");
        std::fs::write(&input1, b"file one").unwrap();
        std::fs::write(&input2, b"file two").unwrap();
        let key = generate_random_key();

        encrypt_file_with_key(input1.to_str().unwrap(), enc1.to_str().unwrap(), &key).unwrap();
        encrypt_file_with_key(input2.to_str().unwrap(), enc2.to_str().unwrap(), &key).unwrap();

        let b1 = std::fs::read(&enc1).unwrap();
        let b2 = std::fs::read(&enc2).unwrap();
        let nonce_offset = 4 + 1 + 8 + 16;
        let n1 = &b1[nonce_offset..nonce_offset + 12];
        let n2 = &b2[nonce_offset..nonce_offset + 12];
        assert_ne!(n1, n2);
        cleanup(&[input1, input2, enc1, enc2]);
    }

    #[test]
    fn wrong_password_verify() {
        let input = tmp("vp_in.txt");
        let enc = tmp("vp.ocrypt");
        std::fs::write(&input, b"verify me").unwrap();

        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "correct").unwrap();
        assert!(verify_password(enc.to_str().unwrap(), "correct").unwrap());
        assert!(!verify_password(enc.to_str().unwrap(), "wrong").unwrap());
        cleanup(&[input, enc]);
    }

    #[test]
    fn secure_delete_removes_file() {
        let p = tmp("sd.txt");
        std::fs::write(&p, b"shred me").unwrap();
        secure_delete(p.to_str().unwrap()).unwrap();
        assert!(!p.exists());
    }

    #[test]
    fn invalid_header_rejected() {
        let bad = tmp("bad.ocrypt");
        std::fs::write(&bad, b"NOPE").unwrap();
        let r = decrypt_file(
            bad.to_str().unwrap(),
            tmp("bad_out").to_str().unwrap(),
            "pw",
        );
        assert!(matches!(r, Err(CryptError::InvalidFormat)));
        cleanup(&[bad]);
    }

    #[test]
    fn version_mismatch_rejected() {
        let input = tmp("ver_in.txt");
        let enc = tmp("ver.ocrypt");
        std::fs::write(&input, b"v").unwrap();
        encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw").unwrap();

        let mut bytes = std::fs::read(&enc).unwrap();
        bytes[4] = 99;
        std::fs::write(&enc, &bytes).unwrap();

        let r = decrypt_file(
            enc.to_str().unwrap(),
            tmp("ver_out").to_str().unwrap(),
            "pw",
        );
        assert!(matches!(r, Err(CryptError::InvalidFormat)));
        cleanup(&[input, enc]);
    }

    #[test]
    fn output_exists_rejected() {
        let input = tmp("oe_in.txt");
        let enc = tmp("oe.ocrypt");
        std::fs::write(&input, b"x").unwrap();
        std::fs::write(&enc, b"y").unwrap();

        let r = encrypt_file(input.to_str().unwrap(), enc.to_str().unwrap(), "pw");
        assert!(matches!(r, Err(CryptError::OutputExists)));
        assert_eq!(std::fs::read(&enc).unwrap(), b"y");
        cleanup(&[input, enc]);
    }

    #[test]
    fn same_path_rejected() {
        let input = tmp("sp.txt");
        std::fs::write(&input, b"x").unwrap();

        let r = encrypt_file(input.to_str().unwrap(), input.to_str().unwrap(), "pw");
        assert!(matches!(r, Err(CryptError::InputInvalid)));
        cleanup(&[input]);
    }
}
