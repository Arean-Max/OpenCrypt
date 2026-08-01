use libloading::Library;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const SUCCESS: i32 = 0;
const INVALID_PASS: i32 = 3;
const INVALID_PARAM: i32 = 5;
const UNSUPPORTED: i32 = 7;
const OUTPUT_EXISTS: i32 = 8;
const INPUT_INVALID: i32 = 9;

struct Ffi {
    encrypt_file: unsafe extern "C" fn(*const i8, *const i8, *const i8) -> i32,
    decrypt_file: unsafe extern "C" fn(*const i8, *const i8, *const i8) -> i32,
    verify_password: unsafe extern "C" fn(*const i8, *const i8) -> i32,
    generate_random_key: unsafe extern "C" fn(*mut i8, i32) -> i32,
    validate_paths: unsafe extern "C" fn(*const i8, *const i8, i32) -> i32,
    register_context_menu: unsafe extern "C" fn(*const i8, *const i8, *const i8, *const i8) -> i32,
    unregister_context_menu: unsafe extern "C" fn() -> i32,
    context_menu_registered: unsafe extern "C" fn(*mut i32) -> i32,
}

impl Ffi {
    fn load() -> (Self, Library) {
        let path = env::var("OPENCRYPT_CORE_DLL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("..\\rust_core\\target\\debug\\rust_core.dll")
            });
        let lib = unsafe { Library::new(&path) }.expect("load rust_core.dll");
        unsafe {
            (
                Ffi {
                    encrypt_file: *lib.get(b"crypt_encrypt_file\0").unwrap(),
                    decrypt_file: *lib.get(b"crypt_decrypt_file\0").unwrap(),
                    verify_password: *lib.get(b"crypt_verify_password\0").unwrap(),
                    generate_random_key: *lib.get(b"crypt_generate_random_key\0").unwrap(),
                    validate_paths: *lib.get(b"crypt_validate_paths\0").unwrap(),
                    register_context_menu: *lib.get(b"crypt_register_context_menu\0").unwrap(),
                    unregister_context_menu: *lib.get(b"crypt_unregister_context_menu\0").unwrap(),
                    context_menu_registered: *lib.get(b"crypt_context_menu_registered\0").unwrap(),
                },
                lib,
            )
        }
    }
}

fn cstr(s: &str) -> Vec<i8> {
    let mut v: Vec<i8> = s.as_bytes().iter().map(|&b| b as i8).collect();
    v.push(0);
    v
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn tmp_path(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "opencrypt_ffi_{}_{}_{}",
        std::process::id(),
        n,
        name
    ))
}

fn with_lib<F: FnOnce(&Ffi)>(f: F) {
    let (ffi, lib) = Ffi::load();
    f(&ffi);
    drop(lib);
}

#[test]
fn ffi_encrypt_decrypt_roundtrip() {
    with_lib(|ffi| {
        let input = tmp_path("in.txt");
        let enc = tmp_path("enc.ocrypt");
        let dec = tmp_path("dec.txt");
        fs::write(&input, vec![7u8; 12345]).unwrap();
        let (i, e, d) = (
            cstr(input.to_str().unwrap()),
            cstr(enc.to_str().unwrap()),
            cstr(dec.to_str().unwrap()),
        );
        let p = cstr("pw123");
        unsafe {
            assert_eq!(
                (ffi.encrypt_file)(i.as_ptr(), e.as_ptr(), p.as_ptr()),
                SUCCESS
            );
            assert_eq!(
                (ffi.decrypt_file)(e.as_ptr(), d.as_ptr(), p.as_ptr()),
                SUCCESS
            );
        }
        assert_eq!(fs::read(&dec).unwrap(), vec![7u8; 12345]);
        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&enc);
        let _ = fs::remove_file(&dec);
    });
}

#[test]
fn ffi_wrong_password_fails() {
    with_lib(|ffi| {
        let input = tmp_path("wp.txt");
        let enc = tmp_path("wp.ocrypt");
        let dec = tmp_path("wp_out.txt");
        fs::write(&input, b"data").unwrap();
        let (i, e, d) = (
            cstr(input.to_str().unwrap()),
            cstr(enc.to_str().unwrap()),
            cstr(dec.to_str().unwrap()),
        );
        let p = cstr("right");
        let bad = cstr("wrong");
        unsafe {
            assert_eq!(
                (ffi.encrypt_file)(i.as_ptr(), e.as_ptr(), p.as_ptr()),
                SUCCESS
            );
            assert_eq!(
                (ffi.decrypt_file)(e.as_ptr(), d.as_ptr(), bad.as_ptr()),
                INVALID_PASS
            );
            assert!(!dec.exists());
        }
        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&enc);
    });
}

#[test]
fn ffi_verify_password_ok_and_fail() {
    with_lib(|ffi| {
        let input = tmp_path("vp.txt");
        let enc = tmp_path("vp.ocrypt");
        fs::write(&input, b"data").unwrap();
        let (i, e) = (cstr(input.to_str().unwrap()), cstr(enc.to_str().unwrap()));
        let p = cstr("pass");
        let bad = cstr("nope");
        unsafe {
            assert_eq!(
                (ffi.encrypt_file)(i.as_ptr(), e.as_ptr(), p.as_ptr()),
                SUCCESS
            );
            assert_eq!((ffi.verify_password)(e.as_ptr(), p.as_ptr()), SUCCESS);
            assert_eq!(
                (ffi.verify_password)(e.as_ptr(), bad.as_ptr()),
                INVALID_PASS
            );
        }
        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&enc);
    });
}

#[test]
fn ffi_generate_random_key_returns_null_terminated_string() {
    with_lib(|ffi| {
        let mut buf = vec![0i8; 64];
        unsafe {
            assert_eq!((ffi.generate_random_key)(buf.as_mut_ptr(), 64), SUCCESS);
        }
        let len = buf.iter().position(|&b| b == 0).expect("null terminator");
        assert_eq!(len, 43);
    });
}

#[test]
fn ffi_generate_random_key_short_buffer_fails() {
    with_lib(|ffi| {
        let mut buf = vec![0i8; 10];
        unsafe {
            assert_eq!(
                (ffi.generate_random_key)(buf.as_mut_ptr(), 10),
                INVALID_PARAM
            );
        }
    });
}

#[test]
fn ffi_null_pointer_is_invalid_param() {
    with_lib(|ffi| unsafe {
        assert_eq!(
            (ffi.encrypt_file)(std::ptr::null(), std::ptr::null(), std::ptr::null()),
            INVALID_PARAM
        );
    });
}

#[test]
fn ffi_validate_paths_codes() {
    with_lib(|ffi| {
        let missing = tmp_path("missing.bin");
        let out = tmp_path("out.ocrypt");
        let (m, o) = (cstr(missing.to_str().unwrap()), cstr(out.to_str().unwrap()));
        unsafe {
            assert_eq!(
                (ffi.validate_paths)(m.as_ptr(), o.as_ptr(), 0),
                INPUT_INVALID
            );
        }
        let input = tmp_path("vin.txt");
        fs::write(&input, b"x").unwrap();
        let (i, o) = (cstr(input.to_str().unwrap()), cstr(out.to_str().unwrap()));
        unsafe {
            assert_eq!((ffi.validate_paths)(i.as_ptr(), o.as_ptr(), 0), SUCCESS);
        }
        fs::write(&out, b"y").unwrap();
        unsafe {
            assert_eq!(
                (ffi.validate_paths)(i.as_ptr(), o.as_ptr(), 0),
                OUTPUT_EXISTS
            );
        }
        let same = cstr(input.to_str().unwrap());
        unsafe {
            assert_eq!(
                (ffi.validate_paths)(same.as_ptr(), same.as_ptr(), 0),
                INPUT_INVALID
            );
        }
        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&out);
    });
}

#[test]
fn ffi_context_menu_register_unregister() {
    with_lib(|ffi| {
        let exe = cstr("C:\\Windows\\System32\\notepad.exe");
        let script = cstr("");
        let enc_label = cstr("Encrypt");
        let dec_label = cstr("Decrypt");
        let code = unsafe {
            (ffi.register_context_menu)(
                exe.as_ptr(),
                script.as_ptr(),
                enc_label.as_ptr(),
                dec_label.as_ptr(),
            )
        };
        if cfg!(windows) {
            assert_eq!(code, SUCCESS);
            let mut out = 0i32;
            unsafe {
                assert_eq!((ffi.context_menu_registered)(&mut out), SUCCESS);
            }
            assert_eq!(out, 1);
            unsafe {
                assert_eq!((ffi.unregister_context_menu)(), SUCCESS);
                assert_eq!((ffi.context_menu_registered)(&mut out), SUCCESS);
            }
            assert_eq!(out, 0);
        } else {
            assert_eq!(code, UNSUPPORTED);
        }
    });
}

#[test]
fn ffi_context_menu_registered_null_out_fails() {
    with_lib(|ffi| unsafe {
        assert_eq!(
            (ffi.context_menu_registered)(std::ptr::null_mut()),
            INVALID_PARAM
        );
    });
}
