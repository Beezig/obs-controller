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
use crate::dialog::{Dialog, AppInfo, DialogResult};
use std::io::{Read, ErrorKind};
use uuid::Uuid;
use std::convert::TryInto;
use std::borrow::Cow;

mod recording;
mod obs;
mod server;
mod verification;
mod dialog;

static mut MODULE: Option<*mut obs_module_t> = None;
const MODULE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "\0");
const MODULE_DESC: &str = concat!(env!("CARGO_PKG_DESCRIPTION"), "\0");

lazy_static::lazy_static! {
    static ref STATE: Arc<Mutex<Option<RecordingState>>> = Arc::new(Mutex::new(None));
    static ref APPS_FILE: String = format!("{}/obs-controller/apps.ock", dirs::data_dir().map(|p| p.to_str().unwrap_or(".").to_string()).unwrap_or_else(|| ".".to_string()));
}

#[derive(serde::Deserialize)]
struct AppRegistrationData {
    uuid: String,
    name: String,
    public_key: String,
}

macro_rules! validate_input {
    ($condition: expr, $desc: literal, $code: literal) => {
        if !($condition) {
            return ($code, Cow::Borrowed(concat!(r#"{"message": ""#, $desc, r#""}"#)));
        }
    };
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
        server.add_route("/register", Box::new(move |mut req| {
            let (tx, rx) = std::sync::mpsc::channel();
            let (status, res): (u16, Cow<str>) = (|| {
                let mut body = String::with_capacity(1024.min(req.body_length().unwrap_or(1024)));
                let body_res = req.as_reader().take(1024).read_to_string(&mut body);
                validate_input!(body_res.is_ok() && !body.is_empty(), "Body cannot be empty", 400);
                let data = serde_json::from_str::<AppRegistrationData>(&body);
                validate_input!(data.is_ok(), "Invalid json data, check the docs for a valid schema", 400);
                let data = data.unwrap();
                validate_input!(data.uuid.len() == 36 || data.uuid.len() == 32, "Invalid UUID. Only stripped and hyphenated UUIDs are supported.", 400);
                validate_input!(!data.name.is_empty() && data.name.len() <= 24, "Name length must be within (0;24]", 400);
                let uuid = Uuid::parse_str(&data.uuid);
                validate_input!(uuid.is_ok(), "Invalid UUID", 400);
                let bytes = base64::decode(&data.public_key);
                validate_input!(bytes.is_ok(), "Invalid Base64", 400);
                let bytes = bytes.unwrap();
                validate_input!(bytes.len() == 32, "Public key must be 32 bytes in length", 400);
                let bytes: [u8; 32] = bytes.try_into().unwrap();
                let pub_key = x25519_dalek::PublicKey::from(bytes);
                Dialog::new(AppInfo::new(data.name), Box::new(tx)).open();
                match rx.recv().unwrap() {
                    DialogResult::Accepted(app) => {
                        let registration = verification::register_encrypt(uuid.unwrap(), app.name.to_str().unwrap().to_string(), pub_key);
                        match registration {
                            Ok((secret, our_pk)) => (200, Cow::Owned(format!(r#"{{"key": "{}", "shared_public": "{}"}}"#, secret, our_pk))),
                            Err(e) if e.kind() == ErrorKind::AlreadyExists => (409, Cow::Borrowed(r#"{"message": "An app with the same UUID already exists"}"#)),
                            Err(e) => {
                                eprintln!("User request refused due to an error {:?}", e);
                                (500, Cow::Borrowed(r#"{"message": "The request couldn't be fulfilled due to an error"}"#))
                            }
                        }
                    }
                    DialogResult::Denied => (401, Cow::Borrowed(r#"{"message": "The registration request was denied by the user"}"#))
                }
            })();
            req.respond(server::json_response(status, &res))
        }));
        server.add_route("/recording/start", Box::new(|mut req| {
            let body = verification::middleware_auth(&mut req).unwrap();
            let (status, msg): (u16, &str) = match body {
                VerificationResult::Body(body) => {
                    let recording = if body.is_empty() { RecordingState::start() } else { RecordingState::start_with_name(body).expect("Couldn't start recording") };
                    *STATE.lock().expect("Poisoned Mutex") = Some(recording);
                    (200, r#"{"message": "Recording started"}"#)
                }
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
                }
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

extern "C" fn on_recording_stopped(event: obs::obs_frontend_event, _private_data: *mut c_void) {
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