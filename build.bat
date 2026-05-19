@echo off
echo Building MythWeaver...

echo [1/4] Building frontend...
cd chronicles\frontend
call npm run build
cd ..\..

echo [2/4] Building backend...
cd chronicles\backend
cargo build --release
cd ..\..

echo [3/4] Building desktop app (generates icons + installer)...
cd chronicles\desktop
cargo tauri icon src-tauri\icons\mythweaver.png
cargo tauri build
cd ..\..

echo Done. Installer in chronicles\desktop\src-tauri\target\release\bundle\