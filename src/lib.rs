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

use std::os::raw::{c_char, c_void};
use obs::obs_module_t;
use std::thread;
use crate::server::HttpServer;
use crate::recording::RecordingState;
use std::sync::{Mutex, Arc};
use std::ptr;
use std::ffi::CStr;
use crate::verification::VerificationResult;

mod recording;
mod obs;
mod server;
mod verification;

static mut MODULE: Option<*mut obs_module_t> = None;
const MODULE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "\0");
const MODULE_DESC: &str = concat!(env!("CARGO_PKG_DESCRIPTION"), "\0");

lazy_static::lazy_static! {
    static ref STATE: Arc<Mutex<Option<RecordingState>>> = Arc::new(Mutex::new(None));
}

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    println!("[OBS Controller] Load started.");
    // Signals
    unsafe { obs::obs_frontend_add_event_callback(Some(on_recording_stopped), ptr::null_mut()); }
    let obs_version = unsafe {
        CStr::from_ptr(obs::obs_get_version_string()).to_str().expect("Invalid version string")
    };

    // Web server
    thread::spawn(move || {
        let mut server = HttpServer::new(8085);
        let info = format!(r#"{{"version": "{}", "obs": "{}"}}"#, env!("CARGO_PKG_VERSION"), obs_version);
        server.add_route("/", Box::new(move |req| {
            req.respond(server::json_response(200, &info))
        }));
        server.add_route("/recording/start", Box::new(|mut req| {
            let body = verification::middleware_auth(&mut req).unwrap();
            let (status, msg): (u16, &str) = match body {
                VerificationResult::Body(body) => {
                    let recording = if body.is_empty() { RecordingState::start() } else { RecordingState::start_with_name(body).expect("Couldn't start recording") };
                    *STATE.lock().expect("Poisoned Mutex") = Some(recording);
                    (200, r#"{"message": "Recording started"}"#)
                },
                VerificationResult::JsonReject(status, msg) => (status, msg)
            };
            req.respond(server::json_response(status, &msg))
        }));
        server.add_route("/recording/stop", Box::new(|mut req| {
            let body = verification::middleware_auth(&mut req).unwrap();
            let (status, msg): (u16, &str) = match body {
                VerificationResult::Body(_) => {
                    unsafe { obs::obs_frontend_recording_stop() };
                    (200, r#"{"message": "Recording stopped"}"#)
                },
                VerificationResult::JsonReject(status, msg) => (status, msg)
            };
            req.respond(server::json_response(status, &msg))
        }));
        server.run().expect("Couldn't run HTTP server.");
    });
    println!("[OBS Controller] Load finished.");
    true
}

#[no_mangle]
pub extern "C" fn obs_module_unload() -> bool {
    println!("[OBS Controller] Unloaded.");
    true
}

extern fn on_recording_stopped(event: obs::obs_frontend_event, _private_data: *mut c_void) {
    if event == obs::obs_frontend_event_OBS_FRONTEND_EVENT_RECORDING_STOPPED {
        let mut lock = STATE.lock().expect("Poisoned Mutex");
        if let Some(state) = lock.take() {
            unsafe { state.revert_name(); }
        }
    }
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