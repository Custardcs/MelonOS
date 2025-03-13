@echo off
REM Simple build script for MelonOS with correct paths

REM Set project root directory
set PROJECT_ROOT=%CD%
echo Project root is: %PROJECT_ROOT%

echo Building UEFI bootloader with nightly toolchain...
cd uefi_bootloader
cargo +nightly build --target x86_64-unknown-uefi --release
if %ERRORLEVEL% neq 0 (
    echo Failed to build bootloader
    cd %PROJECT_ROOT%
    exit /b %ERRORLEVEL%
)
cd %PROJECT_ROOT%

echo Building test kernel...
cd kernel
set ARCH=x86_64

REM Ensure .cargo directory exists
if not exist .cargo mkdir .cargo

REM Use the appropriate config
if "%ARCH%"=="x86_64" (
    echo Using x86_64 config
    if exist .cargo\config.toml.x86_64 (
        copy /Y .cargo\config.toml.x86_64 .cargo\config.toml
    )
    
    cargo +nightly build --release
    if %ERRORLEVEL% neq 0 (
        echo Failed to build kernel
        cd %PROJECT_ROOT%
        exit /b %ERRORLEVEL%
    )
    set KERNEL_NAME=KERNEL_X64.ELF
)
cd %PROJECT_ROOT%

echo Setting up disk image files...
echo Current directory: %CD%

REM Create ESP directory structure with explicit paths
echo Creating ESP directory structure...
if not exist %PROJECT_ROOT%\esp (
    echo Creating esp directory
    mkdir %PROJECT_ROOT%\esp
)

if not exist %PROJECT_ROOT%\esp\EFI (
    echo Creating esp\EFI directory
    mkdir %PROJECT_ROOT%\esp\EFI
)

if not exist %PROJECT_ROOT%\esp\EFI\BOOT (
    echo Creating esp\EFI\BOOT directory
    mkdir %PROJECT_ROOT%\esp\EFI\BOOT
)

if not exist %PROJECT_ROOT%\esp\EFI\KERNEL (
    echo Creating esp\EFI\KERNEL directory
    mkdir %PROJECT_ROOT%\esp\EFI\KERNEL
)

REM Check source files existence
echo Checking source files...
if exist %PROJECT_ROOT%\target\x86_64-unknown-uefi\release\uefi_bootloader.efi (
    echo Bootloader EFI file exists
) else (
    echo ERROR: Bootloader EFI file does not exist at %PROJECT_ROOT%\target\x86_64-unknown-uefi\release\uefi_bootloader.efi!
    exit /b 1
)

if exist %PROJECT_ROOT%\target\x86_64-unknown-none\release\kernel (
    echo Kernel file exists
) else (
    echo ERROR: Kernel file does not exist at %PROJECT_ROOT%\target\x86_64-unknown-none\release\kernel!
    exit /b 1
)

REM Copy files
echo Copying bootloader file...
copy /Y "%PROJECT_ROOT%\target\x86_64-unknown-uefi\release\uefi_bootloader.efi" "%PROJECT_ROOT%\esp\EFI\BOOT\BOOTX64.EFI"

echo Copying kernel file...
copy /Y "%PROJECT_ROOT%\target\x86_64-unknown-none\release\kernel" "%PROJECT_ROOT%\esp\EFI\KERNEL\%KERNEL_NAME%"

REM Check destination files
echo Verifying copied files...
if exist "%PROJECT_ROOT%\esp\EFI\BOOT\BOOTX64.EFI" (
    echo BOOTX64.EFI copied successfully
) else (
    echo ERROR: BOOTX64.EFI not found in destination!
    exit /b 1
)

if exist "%PROJECT_ROOT%\esp\EFI\KERNEL\%KERNEL_NAME%" (
    echo Kernel file copied successfully
) else (
    echo ERROR: Kernel file not found in destination!
    exit /b 1
)

echo Build completed. Files ready at esp\ directory.
qemu-system-x86_64 -drive file=fat:rw:esp,format=raw -bios OVMF.fd -m 128M -debugcon stdio