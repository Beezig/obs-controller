/*
 *  This file is part of OBS Controller.
 *  Copyright (C) 2020 Beezig Team
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::os::raw::c_char;
use obs::obs_module_t;

mod recording;
mod obs;

static mut MODULE: Option<*mut obs_module_t> = None;
const MODULE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "\0");
const MODULE_DESC: &str = concat!(env!("CARGO_PKG_DESCRIPTION"), "\0");

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    println!("[OBS Controller] Load started.");
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
    obs::LIBOBS_API_MAJOR_VER as u32
}