#!/usr/bin/env python3
"""Stage the current Windows x64 build in packages/win64."""
from __future__ import annotations

import json
import hashlib
import os
import shutil
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_DIR = ROOT / "packages" / "win64"
RELEASE_DIR = ROOT / "src-tauri" / "target" / "release"
WEBVIEW2_INSTALLER = "MicrosoftEdgeWebView2RuntimeInstallerX64.exe"
DEPENDENCY_REPORT = "DEPENDENCIES.txt"
CHECKSUM_FILE = "SHA256SUMS.txt"


def fail(message: str) -> None:
    raise SystemExit(message)


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def read_build_version() -> str:
    tauri_version = read_json(ROOT / "src-tauri" / "tauri.conf.json").get("version")
    package_version = read_json(ROOT / "package.json").get("version")
    if tauri_version != package_version:
        fail(f"Version mismatch: package.json has {package_version}, tauri.conf.json has {tauri_version}")
    version_file = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
    if version_file != tauri_version:
        print(f"Warning: VERSION has {version_file}, staging built version {tauri_version}")
    return tauri_version


def require_file(path: Path, description: str) -> Path:
    if not path.is_file():
        fail(f"Missing {description}: {path}")
    return path


def find_one(base: Path, pattern: str, description: str) -> Path:
    matches = sorted(base.glob(pattern), key=lambda path: path.stat().st_mtime, reverse=True)
    if not matches:
        fail(f"Missing {description}: {base / pattern}")
    return matches[0]


def read_pe_imports(path: Path) -> list[str]:
    """Return directly imported DLL names from a PE32/PE32+ executable."""
    data = path.read_bytes()
    if len(data) < 0x40 or data[:2] != b"MZ":
        fail(f"Not a Windows PE executable: {path}")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 24 > len(data) or data[pe_offset : pe_offset + 4] != b"PE\0\0":
        fail(f"Invalid PE header: {path}")

    section_count = struct.unpack_from("<H", data, pe_offset + 6)[0]
    optional_size = struct.unpack_from("<H", data, pe_offset + 20)[0]
    optional_offset = pe_offset + 24
    magic = struct.unpack_from("<H", data, optional_offset)[0]
    if magic == 0x10B:
        data_directory_offset = optional_offset + 96
    elif magic == 0x20B:
        data_directory_offset = optional_offset + 112
    else:
        fail(f"Unsupported PE optional-header format 0x{magic:04x}: {path}")

    import_rva, import_size = struct.unpack_from("<II", data, data_directory_offset + 8)
    if import_rva == 0 or import_size == 0:
        return []

    sections: list[tuple[int, int, int, int]] = []
    section_offset = optional_offset + optional_size
    for index in range(section_count):
        offset = section_offset + index * 40
        if offset + 40 > len(data):
            fail(f"Truncated PE section table: {path}")
        virtual_size, virtual_address, raw_size, raw_offset = struct.unpack_from(
            "<IIII", data, offset + 8
        )
        sections.append((virtual_address, max(virtual_size, raw_size), raw_offset, raw_size))

    def file_offset(rva: int) -> int:
        for virtual_address, mapped_size, raw_offset, raw_size in sections:
            if virtual_address <= rva < virtual_address + mapped_size:
                offset = raw_offset + (rva - virtual_address)
                if offset >= raw_offset + raw_size or offset >= len(data):
                    break
                return offset
        fail(f"Cannot map PE RVA 0x{rva:x} in {path}")
        raise AssertionError("unreachable")

    imports: list[str] = []
    descriptor_offset = file_offset(import_rva)
    while descriptor_offset + 20 <= len(data):
        descriptor = struct.unpack_from("<IIIII", data, descriptor_offset)
        if descriptor == (0, 0, 0, 0, 0):
            break
        name_offset = file_offset(descriptor[3])
        name_end = data.find(b"\0", name_offset)
        if name_end < 0:
            fail(f"Unterminated PE import name in {path}")
        imports.append(data[name_offset:name_end].decode("ascii"))
        descriptor_offset += 20
    return sorted(set(imports), key=str.lower)


def require_windows_gui_subsystem(path: Path) -> None:
    """Fail unless the PE executable is linked as a windowed GUI application."""
    data = path.read_bytes()
    if len(data) < 0x40 or data[:2] != b"MZ":
        fail(f"Not a Windows PE executable: {path}")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    optional_offset = pe_offset + 24
    if (
        optional_offset + 70 > len(data)
        or data[pe_offset : pe_offset + 4] != b"PE\0\0"
    ):
        fail(f"Invalid PE header: {path}")

    subsystem = struct.unpack_from("<H", data, optional_offset + 68)[0]
    windows_gui = 2
    if subsystem != windows_gui:
        fail(
            f"{path.name} uses PE subsystem {subsystem}; expected Windows GUI "
            "(2). A console window would appear when the application starts."
        )


def is_windows_system_dll(name: str) -> bool:
    lower = name.lower()
    if lower.startswith(("api-ms-win-", "ext-ms-win-")):
        return True
    windows_dir = Path(os.environ.get("SystemRoot", r"C:\Windows"))
    return any(
        (windows_dir / directory / name).is_file()
        for directory in ("System32", "SysWOW64")
    )


def runtime_dependencies(exe: Path) -> tuple[list[str], list[Path]]:
    imports = read_pe_imports(exe)
    adjacent = {path.name.lower(): path for path in exe.parent.glob("*.dll")}
    bundled = {path.resolve() for path in adjacent.values()}
    unresolved = []
    for name in imports:
        dependency = adjacent.get(name.lower())
        if dependency:
            bundled.add(dependency.resolve())
        elif not is_windows_system_dll(name):
            unresolved.append(name)
    if unresolved:
        fail(
            "Unresolved non-system runtime DLL imports: "
            + ", ".join(unresolved)
            + ". Place them beside the release executable and configure them in the Tauri bundle."
        )
    return imports, sorted(bundled)


def verify_installer_payload(runtime_dlls: list[Path]) -> None:
    nsis_source = require_file(
        RELEASE_DIR / "nsis" / "x64" / "installer.nsi", "NSIS bundle source"
    )
    nsis_text = nsis_source.read_text(encoding="utf-8")
    if WEBVIEW2_INSTALLER not in nsis_text:
        fail(f"NSIS setup installer does not embed {WEBVIEW2_INSTALLER}")
    missing = [dll.name for dll in runtime_dlls if dll.name not in nsis_text]
    if missing:
        fail(f"NSIS setup installer is missing runtime DLL payloads: {', '.join(missing)}")


def clean_package_dir() -> None:
    target = PACKAGE_DIR.resolve()
    allowed_parent = (ROOT / "packages").resolve()
    if target.parent != allowed_parent or target.name != "win64":
        fail(f"Refusing to clean unexpected package directory: {target}")
    if PACKAGE_DIR.exists():
        shutil.rmtree(PACKAGE_DIR)
    (PACKAGE_DIR / "resources").mkdir(parents=True)


def write_webview2_script() -> None:
    (PACKAGE_DIR / "install-webview2-if-needed.ps1").write_text(
        """$ErrorActionPreference = "Stop"

$installer = Join-Path $PSScriptRoot "MicrosoftEdgeWebView2RuntimeInstallerX64.exe"
$webView2AppGuid = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
$registryPaths = @(
    "HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\$webView2AppGuid",
    "HKLM:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\$webView2AppGuid",
    "HKCU:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\$webView2AppGuid"
)

foreach ($registryPath in $registryPaths) {
    $runtime = Get-ItemProperty -LiteralPath $registryPath -Name "pv" -ErrorAction SilentlyContinue
    if ($runtime.pv) {
        Write-Host "Microsoft Edge WebView2 Runtime already installed: $($runtime.pv)"
        exit 0
    }
}

if (-not (Test-Path -LiteralPath $installer)) {
    throw "Missing WebView2 installer: $installer"
}

Write-Host "Installing Microsoft Edge WebView2 Runtime..."
$process = Start-Process -FilePath $installer -ArgumentList "/silent", "/install" -Wait -PassThru

if ($process.ExitCode -ne 0) {
    throw "WebView2 installer failed with exit code $($process.ExitCode)"
}

Write-Host "Microsoft Edge WebView2 Runtime installed."
""",
        encoding="utf-8",
    )


def write_readme(version: str, nsis_name: str, exe_name: str) -> None:
    (PACKAGE_DIR / "README.txt").write_text(
        f"""QuarterMaster-M {version} Windows x64

Recommended: run {nsis_name}.
The setup installer embeds the full x64 Microsoft Edge WebView2 Runtime offline installer. No internet connection is required, and WebView2 is installed silently only when it is missing.

Portable use: run install-webview2-if-needed.ps1 first if WebView2 may be missing, then run {exe_name}.
See {DEPENDENCY_REPORT} for the audited native runtime dependencies and {CHECKSUM_FILE} for package hashes.
""",
        encoding="utf-8",
    )


def write_dependency_report(imports: list[str], runtime_dlls: list[Path]) -> None:
    bundled = "\n".join(f"  - {path.name}" for path in runtime_dlls) or "  - None"
    imported = "\n".join(f"  - {name} (Windows system component)" for name in imports)
    (PACKAGE_DIR / DEPENDENCY_REPORT).write_text(
        f"""QuarterMaster-M Windows x64 runtime dependency audit

Embedded installer prerequisite:
  - {WEBVIEW2_INSTALLER}
    Included inside the NSIS setup package and also staged beside the portable EXE.

Application-local runtime DLLs:
{bundled}

Direct PE imports:
{imported}

Rust, Tauri, ATASCII, Broadside, and application code are linked into quartermaster-m.exe.
Static .lib files and Rust build artifacts are link-time inputs and are not runtime payloads.
The remaining imports are standard Windows components on supported x64 Windows installations.
""",
        encoding="utf-8",
    )


def write_checksums() -> None:
    rows = []
    for path in sorted(PACKAGE_DIR.rglob("*")):
        if not path.is_file() or path.name == CHECKSUM_FILE:
            continue
        digest = hashlib.sha256()
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
        rows.append(f"{digest.hexdigest()}  {path.relative_to(PACKAGE_DIR).as_posix()}")
    (PACKAGE_DIR / CHECKSUM_FILE).write_text("\n".join(rows) + "\n", encoding="utf-8")


def write_current_version(version: str, exe_name: str, setup_name: str) -> None:
    (ROOT / "current-version").write_text(
        f"{version}:exe:{exe_name}\n{version}:setup:{setup_name}\n",
        encoding="utf-8",
    )


def main() -> None:
    version = read_build_version()
    package_name = read_json(ROOT / "package.json")["name"]
    product_name = read_json(ROOT / "src-tauri" / "tauri.conf.json")["productName"]

    exe = require_file(RELEASE_DIR / f"{package_name}.exe", "release executable")
    require_windows_gui_subsystem(exe)
    nsis = find_one(RELEASE_DIR / "bundle" / "nsis", f"{product_name}_{version}_x64-setup.exe", "NSIS installer")
    webview2 = find_one(RELEASE_DIR, f"**/{WEBVIEW2_INSTALLER}", "WebView2 offline installer")
    icon = require_file(ROOT / "src-tauri" / "icons" / "favicon.ico", "application icon")
    imports, runtime_dlls = runtime_dependencies(exe)
    verify_installer_payload(runtime_dlls)
    portable_name = f"{package_name}-{version}.exe"

    clean_package_dir()
    copied = [
        (exe, PACKAGE_DIR / portable_name),
        (nsis, PACKAGE_DIR / nsis.name),
        (webview2, PACKAGE_DIR / WEBVIEW2_INSTALLER),
        (icon, PACKAGE_DIR / "resources" / "icon.ico"),
    ]
    copied.extend((dll, PACKAGE_DIR / dll.name) for dll in runtime_dlls)
    for source, destination in copied:
        shutil.copy2(source, destination)

    write_webview2_script()
    write_readme(version, nsis.name, portable_name)
    write_dependency_report(imports, runtime_dlls)
    write_checksums()
    write_current_version(version, portable_name, nsis.name)

    print(f"Packaged QuarterMaster-M {version} Windows x64 in {PACKAGE_DIR}")
    for _, destination in copied:
        print(f"  {destination.relative_to(ROOT)}")
    print(f"  {(PACKAGE_DIR / 'install-webview2-if-needed.ps1').relative_to(ROOT)}")
    print(f"  {(PACKAGE_DIR / 'README.txt').relative_to(ROOT)}")
    print(f"  {(PACKAGE_DIR / DEPENDENCY_REPORT).relative_to(ROOT)}")
    print(f"  {(PACKAGE_DIR / CHECKSUM_FILE).relative_to(ROOT)}")


if __name__ == "__main__":
    main()
