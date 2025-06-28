use std::{
    ffi::{c_char, CStr},
    ops::Not,
};

unsafe extern "C" {
    fn getenv(_: *const c_char) -> *const c_char;
}

pub fn contains_var(var: &CStr) -> bool {
    unsafe { getenv(var.as_ptr()) }.is_null().not()
}
