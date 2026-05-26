@echo off
cd /d "%~dp0"
hvigorw clean
hvigorw --mode har
echo.
echo Copying HAR to i3d544...
if exist "hmos_agent_core\build\default\outputs\default\hmos_agent_core.har" (
    copy /Y "hmos_agent_core\build\default\outputs\default\hmos_agent_core.har" "..\..\i3d544-harmony\hap_editor\libs\hmos_agent_core.har"
    echo Done.
) else (
    echo HAR not found. Check build output.
)
