REM Copy the frontend header to the main directory, and rename that to obs.
copy %OBS_PATH%\UI\obs-frontend-api\obs-frontend-api.h %OBS_PATH%\libobs\obs-frontend-api.h /Y
xcopy %OBS_PATH%\libobs %OBS_PATH%\obs /E/H/C/I