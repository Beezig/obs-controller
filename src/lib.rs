use obs_sys::obs_module_t;
use std::os::raw::c_char;

static mut MODULE: Option<*mut obs_module_t> = None;
const MODULE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "\0");
const MODULE_DESC: &str = concat!(env!("CARGO_PKG_DESCRIPTION"), "\0");

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    println!("[OBS Controller] Load finished.");
    true
}

#[no_mangle]
pub extern "C" fn obs_module_unload() -> bool {
    println!("[OBS Controller] Unloaded.");
    true
}

// OBS Module load helpers - these would be declared by the OBS_DECLARE_MODULE() C macro

/// # Safety
/// This is called by OBS before the load function, so nothing else can access the static field.
#[no_mangle]
pub unsafe extern "C" fn obs_module_set_pointer(module: *mut obs_module_t) {
    MODULE = Some(module);
}

#[no_mangle]
pub extern "C" fn obs_module_description() -> *const c_char {
    MODULE_DESC.as_bytes().as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn obs_module_name() -> *const c_char {
    MODULE_NAME.as_bytes().as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn obs_module_ver() -> u32 {
    obs_sys::LIBOBS_API_MAJOR_VER as u32
}