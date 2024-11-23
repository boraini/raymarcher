macro_rules! include_cstr {
    ( $path:literal $(,)? ) => {{
        // Use a constant to force the verification to run at compile time.
        const VALUE: &'static ::core::ffi::CStr = match ::core::ffi::CStr::from_bytes_with_nul(
            concat!(include_str!($path), "\0").as_bytes(),
        ) {
            Ok(value) => value,
            Err(_) => panic!(concat!("interior NUL byte(s) in `", $path, "`")),
        };
        VALUE
    }};
}

pub(crate) use include_cstr;
