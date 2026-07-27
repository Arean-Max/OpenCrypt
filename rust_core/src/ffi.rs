use crate::crypto;
use crate::error::CryptError;
use std::ffi::CStr;
use std::os::raw::c_char;

pub type CryptResult = i32;

const CRYPT_SUCCESS: i32 = 0;
const CRYPT_ERR_IO: i32 = 1;
const CRYPT_ERR_INVALID_FORMAT: i32 = 2;
const CRYPT_ERR_INVALID_PASSWORD: i32 = 3;
const CRYPT_ERR_MEMORY: i32 = 4;
const CRYPT_ERR_INVALID_PARAM: i32 = 5;

fn err_to_code(e: CryptError) -> i32 {
    match e {
        CryptError::Io(_) => CRYPT_ERR_IO,
        CryptError::InvalidFormat => CRYPT_ERR_INVALID_FORMAT,
        CryptError::InvalidPassword => CRYPT_ERR_INVALID_PASSWORD,
        CryptError::Memory => CRYPT_ERR_MEMORY,
        CryptError::InvalidParam => CRYPT_ERR_INVALID_PARAM,
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

fn zeroize_string(s: &mut String) {
    let cap = s.capacity();
    if cap > 0 {
        unsafe { std::ptr::write_bytes(s.as_mut_ptr(), 0, cap); }
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_encrypt_file(
    input_path: *const c_char,
    output_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    let input = match cstr_to_str_safe(input_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let output = match cstr_to_str_safe(output_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let mut pass = match cstr_to_str_safe(password) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let result = crypto::encrypt_file(&input, &output, &pass);
    zeroize_string(&mut pass);
    match result {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_decrypt_file(
    input_path: *const c_char,
    output_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    let input = match cstr_to_str_safe(input_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let output = match cstr_to_str_safe(output_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let mut pass = match cstr_to_str_safe(password) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let result = crypto::decrypt_file(&input, &output, &pass);
    zeroize_string(&mut pass);
    match result {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_verify_password(
    input_path: *const c_char,
    password: *const c_char,
) -> CryptResult {
    let input = match cstr_to_str_safe(input_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let mut pass = match cstr_to_str_safe(password) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let result = crypto::verify_password(&input, &pass);
    zeroize_string(&mut pass);
    match result {
        Ok(true) => CRYPT_SUCCESS,
        Ok(false) => CRYPT_ERR_INVALID_PASSWORD,
        Err(e) => err_to_code(e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_encrypt_file_with_key(
    input_path: *const c_char,
    output_path: *const c_char,
    key_b64: *const c_char,
) -> CryptResult {
    let input = match cstr_to_str_safe(input_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let output = match cstr_to_str_safe(output_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let mut key = match cstr_to_str_safe(key_b64) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let result = crypto::encrypt_file_with_key(&input, &output, &key);
    zeroize_string(&mut key);
    match result {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_decrypt_file_with_key(
    input_path: *const c_char,
    output_path: *const c_char,
    key_b64: *const c_char,
) -> CryptResult {
    let input = match cstr_to_str_safe(input_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let output = match cstr_to_str_safe(output_path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let mut key = match cstr_to_str_safe(key_b64) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    let result = crypto::decrypt_file_with_key(&input, &output, &key);
    zeroize_string(&mut key);
    match result {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crypt_generate_random_key(
    buffer: *mut c_char,
    buffer_len: i32,
) -> CryptResult {
    if buffer.is_null() || buffer_len <= 0 {
        return CRYPT_ERR_INVALID_PARAM;
    }
    let mut key = crypto::generate_random_key();
    let key_bytes = key.as_bytes();
    let len = key_bytes.len().min(buffer_len as usize - 1);
    std::ptr::copy_nonoverlapping(key_bytes.as_ptr(), buffer as *mut u8, len);
    std::ptr::write(buffer.add(len) as *mut u8, 0);
    zeroize_string(&mut key);
    CRYPT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn crypt_secure_delete(path: *const c_char) -> CryptResult {
    let path = match cstr_to_str_safe(path) {
        Ok(s) => s,
        Err(e) => return err_to_code(e),
    };
    match crypto::secure_delete(&path) {
        Ok(()) => CRYPT_SUCCESS,
        Err(e) => err_to_code(e),
    }
}
