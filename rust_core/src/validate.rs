use crate::error::{CryptError, Result};
use std::path::{Component, Path};

const OCRYPT_EXT: &str = "ocrypt";

pub fn validate_paths(input: &str, output: &str, is_decrypt: bool) -> Result<()> {
    let in_path = Path::new(input);
    let out_path = Path::new(output);
    let meta = match std::fs::metadata(input) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CryptError::InputInvalid);
        }
        Err(e) => return Err(CryptError::Io(e)),
    };
    if !meta.is_file() {
        return Err(CryptError::InputInvalid);
    }
    if same_path(in_path, out_path) {
        return Err(CryptError::InputInvalid);
    }
    if out_path.exists() {
        return Err(CryptError::OutputExists);
    }
    let ext = in_path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if is_decrypt {
        if ext != OCRYPT_EXT {
            return Err(CryptError::InputInvalid);
        }
    } else if ext == OCRYPT_EXT {
        return Err(CryptError::InputInvalid);
    }
    match out_path.parent() {
        Some(p) if p.as_os_str().is_empty() || p.is_dir() => {}
        _ => return Err(CryptError::InputInvalid),
    }
    Ok(())
}

pub(crate) fn same_path(a: &Path, b: &Path) -> bool {
    norm(a) == norm(b)
}

fn norm(p: &Path) -> String {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(p),
            Err(_) => p.to_path_buf(),
        }
    };
    let mut parts: Vec<String> = Vec::new();
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            other => parts.push(other.as_os_str().to_string_lossy().to_ascii_lowercase()),
        }
    }
    parts.join("\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "opencrypt_validate_{}_{}",
            std::process::id(),
            name
        ));
        p
    }

    #[test]
    fn valid_encrypt_and_decrypt_paths() {
        let input = tmp("ok.bin");
        let enc = tmp("ok.ocrypt");
        std::fs::write(&input, b"x").unwrap();
        assert!(validate_paths(input.to_str().unwrap(), enc.to_str().unwrap(), false).is_ok());
        std::fs::write(&enc, b"y").unwrap();
        assert!(validate_paths(
            enc.to_str().unwrap(),
            tmp("ok2.bin").to_str().unwrap(),
            true
        )
        .is_ok());
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&enc);
    }

    #[test]
    fn missing_input_rejected() {
        let r = validate_paths(
            tmp("nope.bin").to_str().unwrap(),
            tmp("out.ocrypt").to_str().unwrap(),
            false,
        );
        assert!(matches!(r, Err(CryptError::InputInvalid)));
    }

    #[test]
    fn directory_input_rejected() {
        let r = validate_paths(".", tmp("out.ocrypt").to_str().unwrap(), false);
        assert!(matches!(r, Err(CryptError::InputInvalid)));
    }

    #[test]
    fn existing_output_rejected() {
        let input = tmp("in.bin");
        let out = tmp("out.ocrypt");
        std::fs::write(&input, b"x").unwrap();
        std::fs::write(&out, b"y").unwrap();
        let r = validate_paths(input.to_str().unwrap(), out.to_str().unwrap(), false);
        assert!(matches!(r, Err(CryptError::OutputExists)));
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn same_path_rejected() {
        let input = tmp("same.bin");
        std::fs::write(&input, b"x").unwrap();
        let r = validate_paths(input.to_str().unwrap(), input.to_str().unwrap(), false);
        assert!(matches!(r, Err(CryptError::InputInvalid)));
        let _ = std::fs::remove_file(&input);
    }

    #[test]
    fn wrong_extension_rejected() {
        let a = tmp("a.txt");
        std::fs::write(&a, b"x").unwrap();
        assert!(matches!(
            validate_paths(a.to_str().unwrap(), tmp("b.ocrypt").to_str().unwrap(), true),
            Err(CryptError::InputInvalid)
        ));
        let oc = tmp("a.ocrypt");
        std::fs::write(&oc, b"y").unwrap();
        assert!(matches!(
            validate_paths(oc.to_str().unwrap(), tmp("c.bin").to_str().unwrap(), false),
            Err(CryptError::InputInvalid)
        ));
        let _ = std::fs::remove_file(&a);
        let _ = std::fs::remove_file(&oc);
    }

    #[test]
    fn missing_output_parent_rejected() {
        let input = tmp("c.bin");
        std::fs::write(&input, b"x").unwrap();
        let mut missing_dir = std::env::temp_dir();
        missing_dir.push(format!("opencrypt_nodir_{}", std::process::id()));
        let out = missing_dir.join("out.ocrypt");
        let r = validate_paths(input.to_str().unwrap(), out.to_str().unwrap(), false);
        assert!(matches!(r, Err(CryptError::InputInvalid)));
        let _ = std::fs::remove_file(&input);
    }
}
