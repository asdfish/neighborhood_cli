use std::ffi::{CStr, OsStr, c_char};
unsafe extern "C" {
    fn getenv(_: *const c_char) -> *const c_char;
}

/// # Safety
///
/// You should not call this when there are multiple threads since it can lead to a race condition.
pub unsafe fn var(var: &CStr) -> Option<&'static OsStr> {
    let var = unsafe { getenv(var.as_ptr()) };
    if var.is_null() {
        None
    } else {
        Some(unsafe { OsStr::from_encoded_bytes_unchecked(CStr::from_ptr(var).to_bytes()) })
    }
}
