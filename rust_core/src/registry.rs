use crate::error::{CryptError, Result};

#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
const ENCRYPT_GUID: &str = "{8A2C5B1E-3D4F-4A6B-9C8D-1E2F3A4B5C6D}";
#[cfg(windows)]
const DECRYPT_GUID: &str = "{9B3D6C2F-4E5A-5B7C-0D9E-2F3A4B5C6D7E}";

#[cfg(windows)]
fn build_cmd(exe: &str, script: &str, flag: &str) -> String {
    if script.is_empty() {
        format!("\"{exe}\" --{flag} \"%1\"")
    } else {
        format!("\"{exe}\" \"{script}\" --{flag} \"%1\"")
    }
}

#[cfg(windows)]
pub fn register(exe: &str, script: &str, encrypt_label: &str, decrypt_label: &str) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    if !Path::new(exe).exists() {
        return Err(CryptError::InputInvalid);
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (classes, _) = hkcu
        .create_subkey("Software\\Classes")
        .map_err(CryptError::Io)?;

    let (enc_root, _) = classes
        .create_subkey(format!("*\\shell\\{ENCRYPT_GUID}"))
        .map_err(CryptError::Io)?;
    enc_root
        .set_value("", &encrypt_label)
        .map_err(CryptError::Io)?;
    enc_root.set_value("Icon", &exe).map_err(CryptError::Io)?;
    enc_root
        .set_value("Exclude", &".ocrypt")
        .map_err(CryptError::Io)?;
    let (enc_cmd, _) = enc_root.create_subkey("command").map_err(CryptError::Io)?;
    enc_cmd
        .set_value("", &build_cmd(exe, script, "encrypt"))
        .map_err(CryptError::Io)?;

    let (dec_root, _) = classes
        .create_subkey(format!(".ocrypt\\shell\\{DECRYPT_GUID}"))
        .map_err(CryptError::Io)?;
    dec_root
        .set_value("", &decrypt_label)
        .map_err(CryptError::Io)?;
    dec_root.set_value("Icon", &exe).map_err(CryptError::Io)?;
    let (dec_cmd, _) = dec_root.create_subkey("command").map_err(CryptError::Io)?;
    dec_cmd
        .set_value("", &build_cmd(exe, script, "decrypt"))
        .map_err(CryptError::Io)?;

    Ok(())
}

#[cfg(not(windows))]
pub fn register(
    _exe: &str,
    _script: &str,
    _encrypt_label: &str,
    _decrypt_label: &str,
) -> Result<()> {
    Err(CryptError::Unsupported)
}

#[cfg(windows)]
pub fn unregister() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey("Software\\Classes")
        .map_err(CryptError::Io)?;
    for guid in [ENCRYPT_GUID, DECRYPT_GUID] {
        for prefix in ["*\\shell", ".ocrypt\\shell"] {
            let key = format!("{prefix}\\{guid}");
            match classes.open_subkey(&key) {
                Ok(k) => match k.delete_subkey_all("command") {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(CryptError::Io(e)),
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(CryptError::Io(e)),
            }
            match classes.delete_subkey(&key) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(CryptError::Io(e)),
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn unregister() -> Result<()> {
    Err(CryptError::Unsupported)
}

#[cfg(windows)]
pub fn is_registered() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = format!("Software\\Classes\\*\\shell\\{ENCRYPT_GUID}\\command");
    hkcu.open_subkey(path).is_ok()
}

#[cfg(not(windows))]
pub fn is_registered() -> bool {
    false
}
