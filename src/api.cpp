// Copyright (C) 2020 Beezig Team
//
// This file is part of obs-controller.
//
// obs-controller is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// obs-controller is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with obs-controller.  If not, see <http://www.gnu.org/licenses/>.

#include "api.hpp"
#include <obs-frontend-api.h>
#include <obs.hpp>
#include <util/config-file.h>

void ObsController::StartRecording()
{
    obs_frontend_recording_start();
}

void ObsController::StopRecording()
{
    obs_frontend_recording_stop();
}

void ObsController::SetFileName(const char *name)
{
    auto profile = obs_frontend_get_profile_config();
    config_set_string(profile, "Output", "FilenameFormatting", name);
    config_save(profile);
}