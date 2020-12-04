macro_rules! println {
    () => (println!(""));
    ($($arg:tt)*) => ({
        let c_str = std::ffi::CString::new(format!($($arg)*)).expect("Couldn't create C-string");
        unsafe { $crate::obs::blog($crate::obs::LOG_INFO as i32, c_str.as_ptr()); }
    })
}

macro_rules! eprintln {
    () => (eprintln!(""));
    ($($arg:tt)*) => ({
        let c_str = std::ffi::CString::new(format!($($arg)*)).expect("Couldn't create C-string");
        unsafe { $crate::obs::blog($crate::obs::LOG_ERROR as i32, c_str.as_ptr()); }
    })
}

#[allow(unused_macros)]
macro_rules! debug {
    () => (debug!(""));
    ($($arg:tt)*) => ({
        let c_str = std::ffi::CString::new(format!($($arg)*)).expect("Couldn't create C-string");
        unsafe { $crate::obs::blog($crate::obs::LOG_DEBUG as i32, c_str.as_ptr()); }
    })
}
