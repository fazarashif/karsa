# Sets up the MSVC build environment for Rust/Tauri in the current PowerShell session.
#
# Why this exists: on this laptop Visual Studio 2022 has the C++ compiler and the Windows
# SDK, but not the standard x64 C runtime libraries, and installing them needs admin rights.
# The OneCore x64 libraries that are installed provide the same C runtime (they link the
# same vcruntime/UCRT DLLs), so the linker is pointed at those instead.
#
# Usage, from the repository root:
#   . .\scripts\msvc-env.ps1
#   npm run tauri dev
#
# Once the "MSVC v143 x64/x86 build tools" component is installed properly, this script
# is no longer needed.

$vs = 'C:\Program Files\Microsoft Visual Studio\2022\Community'
$msvc = Get-ChildItem "$vs\VC\Tools\MSVC" -Directory | Sort-Object Name -Descending | Select-Object -First 1
if (-not $msvc) { throw "No MSVC toolset found under $vs\VC\Tools\MSVC" }

$sdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10"
$sdk = Get-ChildItem "$sdkRoot\Lib" -Directory |
  Where-Object { Test-Path "$($_.FullName)\um\x64\kernel32.lib" } |
  Sort-Object Name -Descending | Select-Object -First 1
if (-not $sdk) { throw "No Windows SDK with x64 libraries found under $sdkRoot\Lib" }
$sdkVer = $sdk.Name

$crt = "$($msvc.FullName)\lib\x64"
if (-not (Test-Path "$crt\msvcrt.lib")) { $crt = "$($msvc.FullName)\lib\onecore\x64" }

# Rust's linker discovery trusts a developer-prompt environment when these are set.
$env:VSINSTALLDIR = "$vs\"
$env:VCINSTALLDIR = "$vs\VC\"
$env:VCToolsInstallDir = "$($msvc.FullName)\"
$env:VSCMD_ARG_HOST_ARCH = 'x64'
$env:VSCMD_ARG_TGT_ARCH = 'x64'

$env:PATH = @(
  "$($msvc.FullName)\bin\Hostx64\x64",
  "$sdkRoot\bin\$sdkVer\x64",
  "$env:USERPROFILE\.cargo\bin",
  $env:PATH
) -join ';'
$env:LIB = @($crt, "$sdkRoot\Lib\$sdkVer\ucrt\x64", "$sdkRoot\Lib\$sdkVer\um\x64") -join ';'
$env:INCLUDE = @(
  "$($msvc.FullName)\include",
  "$sdkRoot\Include\$sdkVer\ucrt",
  "$sdkRoot\Include\$sdkVer\um",
  "$sdkRoot\Include\$sdkVer\shared",
  "$sdkRoot\Include\$sdkVer\winrt"
) -join ';'

Write-Host "MSVC $($msvc.Name), Windows SDK $sdkVer, CRT from $crt"
