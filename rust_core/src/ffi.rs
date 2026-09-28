use crate::error::CryptError;
use crate::{crypto, registry, validate};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use zeroize::Zeroize;

pub type CryptResult = i32;

const CRYPT_SUCCESS: i32 = 0;
const CRYPT_ERR_IO: i32 = 1;
const CRYPT_ERR_INVALID_FORMAT: i32 = 2;
const CRYPT_ERR_INVALID_PASSWORD: i32 = 3;
const CRYPT_ERR_MEMORY: i32 = 4;
const CRYPT_ERR_INVALID_PARAM: i32 = 5;
const CRYPT_ERR_PANIC: i32 = 6;
const CRYPT_ERR_UNSUPPORTED: i32 = 7;
const CRYPT_ERR_OUTPUT_EXISTS: i32 = 8;
const CRYPT_ERR_INPUT_INVALID: i32 = 9;

fn err_to_code(e: CryptError) -> i32 {
    match e {
        CryptError::Io(_) => CRYPT_ERR_IO,
        CryptError::InvalidFormat => CRYPT_ERR_INVALID_FORMAT,
        CryptError::InvalidPassword => CRYPT_ERR_INVALID_PASSWORD,
        CryptError::Memory => CRYPT_ERR_MEMORY,
        CryptError::InvalidParam => CRYPT_ERR_INVALID_PARAM,
        CryptError::Unsupported => CRYPT_ERR_UNSUPPORTED,
        CryptError::OutputExists => CRYPT_ERR_OUTPUT_EXISTS,
        CryptError::InputInvalid => CRYPT_ERR_INPUT_INVALID,
    }
}

fn guard(f: impl FnOnce() -> CryptResult) -> CryptResult {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(code) => code,
        Err(_) => CRYPT_ERR_PANIC,
    }
}

fn cstr_to_str_safe(ptr: *const c_char) -> Result<String, CryptError> {
    if ptr.is_null() {
        return Err(CryptError::InvalidParam);
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_owned())
            .map_err(|_| CryptError::InvalidParam)
    }
}

fn cstrs(
    a: *const c_char,
    b: *const c_char,
    c: *const c_char,
) -> Result<(String, String, String), CryptError> {
    Ok((
        cstr_to_str_safe(a)?,
        cstr_to_str_safe(b)?,
        cstr_to_str_safe(c)?,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn crypt_encrypt_file(
    input_path: *const c_char,
    output_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    guard(|| {
        let (input, output, mut pass) = match cstrs(input_path, output_path, password) {
            Ok(t) => t,
            Err(e) => return err_to_code(e),
        };
        let result = crypto::encrypt_file(&input, &output, &pass);
        pass.zeroize();
        match result {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_decrypt_file(
    input_path: *const c_char,
    output_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    guard(|| {
        let (input, output, mut pass) = match cstrs(input_path, output_path, password) {
            Ok(t) => t,
            Err(e) => return err_to_code(e),
        };
        let result = crypto::decrypt_file(&input, &output, &pass);
        pass.zeroize();
        match result {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_verify_password(
    input_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    guard(|| {
        let (input, mut pass) = match (cstr_to_str_safe(input_path), cstr_to_str_safe(password)) {
            (Ok(i), Ok(p)) => (i, p),
            _ => return CRYPT_ERR_INVALID_PARAM,
        };
        let result = crypto::verify_password(&input, &pass);
        pass.zeroize();
        match result {
            Ok(true) => CRYPT_SUCCESS,
            Ok(false) => CRYPT_ERR_INVALID_PASSWORD,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_encrypt_file_with_key(
    input_path: *const c_char,
    output_path: *const c_char,
    key_b64: *const c_char,
) -> CryptResult {
    guard(|| {
        let (input, output, mut key) = match cstrs(input_path, output_path, key_b64) {
            Ok(t) => t,
            Err(e) => return err_to_code(e),
        };
        let result = crypto::encrypt_file_with_key(&input, &output, &key);
        key.zeroize();
        match result {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_decrypt_file_with_key(
    input_path: *const c_char,
    output_path: *const c_char,
    key_b64: *const c_char,
) -> CryptResult {
    guard(|| {
        let (input, output, mut key) = match cstrs(input_path, output_path, key_b64) {
            Ok(t) => t,
            Err(e) => return err_to_code(e),
        };
        let result = crypto::decrypt_file_with_key(&input, &output, &key);
        key.zeroize();
        match result {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_generate_random_key(
    buffer: *mut c_char,
    buffer_len: i32,
) -> CryptResult {
    guard(|| {
        if buffer.is_null() || buffer_len < 44 {
            return CRYPT_ERR_INVALID_PARAM;
        }
        let key = crypto::generate_random_key();
        let key_bytes = key.as_bytes();
        std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), buffer as *mut u8, key_bytes.len());
        std::ptr::write(buffer.add(key_bytes.len()) as *mut u8, 0);
        CRYPT_SUCCESS
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_secure_delete(path: *const c_char) -> CryptResult {
    guard(|| {
        let path = match cstr_to_str_safe(path) {
            Ok(s) => s,
            Err(e) => return err_to_code(e),
        };
        match crypto::secure_delete(&path) {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_validate_paths(
    input_path: *const c_char,
    output_path: *const c_char,
    is_decrypt: i32,
) -> CryptResult {
    guard(|| {
        let (input, output) = match (cstr_to_str_safe(input_path), cstr_to_str_safe(output_path)) {
            (Ok(i), Ok(o)) => (i, o),
            _ => return CRYPT_ERR_INVALID_PARAM,
        };
        match validate::validate_paths(&input, &output, is_decrypt != 0) {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_register_context_menu(
    exe_path: *const c_char,
    script_path: *const c_char,
    encrypt_label: *const c_char,
    decrypt_label: *const c_char,
) -> CryptResult {
    guard(|| {
        let (exe, script, enc_label, dec_label) = match (
            cstr_to_str_safe(exe_path),
            cstr_to_str_safe(script_path),
            cstr_to_str_safe(encrypt_label),
            cstr_to_str_safe(decrypt_label),
        ) {
            (Ok(a), Ok(b), Ok(c), Ok(d)) => (a, b, c, d),
            _ => return CRYPT_ERR_INVALID_PARAM,
        };
        match registry::register(&exe, &script, &enc_label, &dec_label) {
            Ok(()) => CRYPT_SUCCESS,
            Err(e) => err_to_code(e),
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_unregister_context_menu() -> CryptResult {
    guard(|| match registry::unregister() {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    })
}

#[no_mangle]
pub unsafe extern "C" fn crypt_context_menu_registered(out: *mut i32) -> CryptResult {
    guard(|| {
        if out.is_null() {
            return CRYPT_ERR_INVALID_PARAM;
        }
        let registered = registry::is_registered();
        unsafe { std::ptr::write(out, if registered { 1 } else { 0 }) };
        CRYPT_SUCCESS
    })
}
