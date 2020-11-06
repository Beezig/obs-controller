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

use std::ffi::{CStr, CString};
use std::str::Utf8Error;

use crate::obs;

pub enum RecordingState {
    Regular,
    CustomName(String)
}

impl RecordingState {
    /// Starts recording with the default file name formatting.
    pub fn start() -> RecordingState {
        unsafe { obs::obs_frontend_recording_start() }
        RecordingState::Regular
    }

    /// Starts recording, setting a custom file name formatting.
    /// # Arguments
    /// * `name` - The file name format for the video output. Refer to the OBS documentation for placeholders.
    ///
    /// # Errors
    /// `Utf8Error` if the old name can't be converted to a valid UTF-8 string.
    ///
    /// # Example
    /// ```no_run
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(2000));
    ///     let recording = RecordingState::start_with_name(String::from("Test Recording Name")).unwrap();
    ///     thread::sleep(Duration::from_millis(2000));
    ///     recording.stop();
    /// });
    /// ```
    pub fn start_with_name(name: String) -> Result<RecordingState, Utf8Error> {
        unsafe {
            // Before starting the recording, we set the new file name and store the old one.
            let config = obs::obs_frontend_get_profile_config();
            let name = CString::new(name).expect("Converting name to CString");
            let output = CString::new("Output").expect("Output as CString");
            let formatting = CString::new("FilenameFormatting").expect("FNFmt as CString");
            let old_name = obs::config_get_string(config, output.as_ptr(), formatting.as_ptr());
            obs::config_set_string(config, output.as_ptr(), formatting.as_ptr(), name.as_ptr());
            obs::config_save(config);
            obs::obs_frontend_recording_start();
            Ok(RecordingState::CustomName(CStr::from_ptr(old_name).to_str()?.to_string()))
        }
    }

    /// Reverts the file name formatting to the saved one.
    pub(crate) unsafe fn revert_name(&self) {
        if let RecordingState::CustomName(old_name) = self {
            // We set the old name back
            let config = obs::obs_frontend_get_profile_config();
            let name = CString::new(old_name.as_str()).expect("Converting old name to CString");
            let output = CString::new("Output").expect("Output as CString");
            let formatting = CString::new("FilenameFormatting").expect("FNFmt as CString");
            obs::config_set_string(config, output.as_ptr(), formatting.as_ptr(), name.as_ptr());
            obs::config_save(config);
        }
    }

    /// Stops the current recording.
    pub fn stop(&self) {
        unsafe {
            obs::obs_frontend_recording_stop()
        }
    }
}

impl Drop for RecordingState {
    fn drop(&mut self) {
        self.stop();
    }
}