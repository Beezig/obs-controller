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
use std::thread;
use crate::server::HttpServer;
use tiny_http::{Response, StatusCode};
use std::io::Read;
use crate::recording::RecordingState;
use std::sync::{RwLock, Arc};

mod recording;
mod obs;
mod server;

static mut MODULE: Option<*mut obs_module_t> = None;
const MODULE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "\0");
const MODULE_DESC: &str = concat!(env!("CARGO_PKG_DESCRIPTION"), "\0");

lazy_static::lazy_static! {
    static ref STATE: Arc<RwLock<Option<RecordingState>>> = Arc::new(RwLock::new(None));
}

#[no_mangle]
pub extern "C" fn obs_module_load() -> bool {
    println!("[OBS Controller] Load started.");
    thread::spawn(move || {
        let mut server = HttpServer::new(8085);
        server.add_route("/", Box::new(|req| req.respond(Response::new_empty(StatusCode(200)))));
        server.add_route("/recording/start", Box::new(|mut req| {
            let mut body = String::with_capacity(1024.min(req.body_length().unwrap_or(1024)));
            req.as_reader().take(1024).read_to_string(&mut body).expect("Couldn't read body.");
            let recording = if body.is_empty() { RecordingState::start() } else { RecordingState::start_with_name(body).expect("Couldn't start recording") };
            *STATE.write().expect("Poisoned RwLock") = Some(recording);
            req.respond(Response::new_empty(StatusCode(200)))
        }));
        server.add_route("/recording/stop", Box::new(|req| {
            match &mut *STATE.write().expect("Poisoned RwLock") {
                opt @ Some(_) => drop(opt.take()),
                None => unsafe { obs::obs_frontend_recording_stop() }
            }
            req.respond(Response::new_empty(StatusCode(200)))
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