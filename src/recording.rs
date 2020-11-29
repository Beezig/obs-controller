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
    pub fn stop() -> StopResponse {
        unsafe {
            let path = RecordingState::recording_path();
            obs::obs_frontend_recording_stop();
            StopResponse {path}
        }
    }

    unsafe fn recording_path() -> Option<String> {
        let output = obs::obs_frontend_get_recording_output();
        if output.is_null() {
            return None;
        }
        let output = FileOutput(output);
        let settings = obs::obs_output_get_settings(output.0);
        if settings.is_null() {
            return None;
        }
        let settings = OutputData(settings);
        let mut path = obs::obs_data_get_string(settings.0, "path\0".as_ptr() as *const i8);
        if path.is_null() {
            path = obs::obs_data_get_string(settings.0, "url\0".as_ptr() as *const i8);
        }
        if path.is_null() {
            return None;
        }
        let cstr = CStr::from_ptr(path);
        let parsed = cstr.to_str().unwrap();
        if parsed.is_empty() { None } else { Some(parsed.to_string()) }
    }
}

#[derive(serde::Serialize)]
pub struct StopResponse {
    path: Option<String>
}

struct FileOutput(*mut obs::obs_output_t);
struct OutputData(*mut obs::obs_data_t);

impl Drop for FileOutput {
    fn drop(&mut self) {
        unsafe { obs::obs_output_release(self.0); }
    }
}

impl Drop for OutputData {
    fn drop(&mut self) {
        unsafe { obs::obs_data_release(self.0); }
    }
}