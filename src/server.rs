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

use std::collections::HashMap;
use std::error::Error;
use std::io;

use tiny_http::{Request, Response, Server, StatusCode};

pub type ResCallback = Box<dyn Fn(Request) -> io::Result<()>>;

pub struct HttpServer {
    port: u16,
    router: HashMap<&'static str, ResCallback>
}

impl HttpServer {
    pub fn new(port: u16) -> HttpServer {
        HttpServer { port, router: HashMap::new() }
    }

    pub fn add_route(&mut self, path: &'static str, callback: ResCallback) {
        self.router.insert(path, callback);
    }

    pub fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let server = Server::http(&format!("127.0.0.1:{}", self.port))?;
        for request in server.incoming_requests() {
            match self.router.get(request.url()) {
                Some(route) => route(request),
                None => request.respond(Response::new_empty(StatusCode(404)))
            }?;
        }
        Ok(())
    }
}