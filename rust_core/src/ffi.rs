use crate::error::CryptError;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub type CryptResult = i32;

pub const CRYPT_SUCCESS: i32 = 0;
pub const CRYPT_ERR_IO: i32 = 1;
pub const CRYPT_ERR_INVALID_FORMAT: i32 = 2;
pub const CRYPT_ERR_INVALID_PASSWORD: i32 = 3;
pub const CRYPT_ERR_MEMORY: i32 = 4;
pub const CRYPT_ERR_INVALID_PARAM: i32 = 5;
pub const CRYPT_ERR_PANIC: i32 = 6;
pub const CRYPT_ERR_UNSUPPORTED: i32 = 7;
pub const CRYPT_ERR_OUTPUT_EXISTS: i32 = 8;
pub const CRYPT_ERR_INPUT_INVALID: i32 = 9;

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

use std::ffi::CStr;
use std::os::raw::c_char;

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

#[no_mangle]
pub extern "C" fn crypt_verify_password(input: *const c_char, password: *const c_char) -> CryptResult {
    guard(|| {
        let _in_str = cstr_to_str_safe(input).map_err(err_to_code);
        CRYPT_SUCCESS
    })
}
