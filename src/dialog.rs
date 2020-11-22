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
use cpp::cpp;

use qt_core::QString;
use qt_widgets::QMessageBox;
use qt_widgets::q_message_box::StandardButton;
use std::ffi::CString;
use std::sync::mpsc::Sender;

cpp!{{
    #include <QtCore/QCoreApplication>
    #include <QtCore/QMetaObject>
    #include <QtCore/QString>
    #include <QtWidgets/QMessageBox>
}}

#[derive(Debug)]
pub enum DialogResult {
    Accepted(Box<AppInfo>),
    Denied
}

#[repr(C)]
pub struct Dialog {
    app: Box<AppInfo>,
    sender: Box<Sender<DialogResult>>
}

#[repr(C)]
#[derive(Debug)]
pub struct AppInfo {
    name: CString
}

impl AppInfo {
    pub fn new(name: String) -> Box<AppInfo> {
        Box::new(Self {name: CString::new(name).unwrap()})
    }
}

impl Dialog {
    pub fn new(app: Box<AppInfo>, sender: Box<Sender<DialogResult>>) -> Self {
        Dialog {app, sender}
    }

    pub fn open(self) {
        unsafe {
            let app_info = Box::into_raw(self.app);
            let sender = Box::into_raw(self.sender);
            // Open on main thread
            cpp!([app_info as "void*", sender as "void*"] {
                QCoreApplication *app = QCoreApplication::instance();
                QMetaObject::invokeMethod(app, [=] {
                    rust!(Dialog_Callback [app_info: *mut AppInfo as "void*", sender: *mut Sender<DialogResult> as "void*"] {
                        open_dialog(Box::from_raw(app_info), Box::from_raw(sender));
                    });
                });
            });
        }
    }
}

#[allow(clippy::boxed_local)]
fn open_dialog(app_info: Box<AppInfo>, sender: Box<Sender<DialogResult>>) {
    let res = unsafe {
        let msg_box = QMessageBox::new();
        let std = app_info.name.to_str().unwrap();
        let title = QString::from_std_str(&std);
        msg_box.set_window_title(&title);
        let desc = QString::from_std_str(&format!("Allow {} to access OBS?", std));
        msg_box.set_text(&desc);
        msg_box.add_button_standard_button(StandardButton::Yes);
        msg_box.add_button_standard_button(StandardButton::No);
        msg_box.set_default_button_standard_button(StandardButton::No);
        msg_box.set_escape_button_standard_button(StandardButton::No);
        msg_box.exec()
    };
    sender.send(if res == 65536 {DialogResult::Denied} else {DialogResult::Accepted(app_info)}).unwrap();
}