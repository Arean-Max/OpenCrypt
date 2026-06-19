use crate::error::CryptError;

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
