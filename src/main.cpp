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

#include <obs-module.h>
#include <obs-frontend-api.h>
#include <iostream>
#include "api.hpp"

using namespace std;

OBS_DECLARE_MODULE()

extern "C"
{
    bool obs_module_load()
    {
        cout << "[Beezig] Loading OBS Controller" << endl;
        auto eventCallback = [](enum obs_frontend_event event, void *param) {
            if (event == OBS_FRONTEND_EVENT_FINISHED_LOADING)
            {
                // Run any frontend load events here
                obs_frontend_remove_event_callback((obs_frontend_event_cb)param, nullptr);
            }
        };
        obs_frontend_add_event_callback(eventCallback, (void *)(obs_frontend_event_cb)eventCallback);
        return true;
    }

    void obs_module_unload()
    {
    }
}