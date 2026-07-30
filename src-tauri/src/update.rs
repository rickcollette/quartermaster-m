use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::version::APP_VERSION;

const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/rickcollette/quartermaster-m/refs/heads/main/current-version";
const RELEASE_DOWNLOAD_ROOT: &str =
    "https://github.com/rickcollette/quartermaster-m/releases/download";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_UPDATE_BYTES: u64 = 750 * 1024 * 1024;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq)]
struct Manifest {
    version: String,
    exe_file: Option<String>,
    msi_file: Option<String>,
    dmg_file: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    current_version: String,
    latest_version: String,
    state: String,
    platform: String,
    exe_file: Option<String>,
    msi_file: Option<String>,
    dmg_file: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownload {
    version: String,
    path: String,
    launched: bool,
}

fn parse_version(value: &str) -> Result<[u64; 3]> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(format!("invalid application version {value:?}"));
    }
    Ok([
        parts[0]
            .parse()
            .map_err(|_| format!("invalid application version {value:?}"))?,
        parts[1]
            .parse()
            .map_err(|_| format!("invalid application version {value:?}"))?,
        parts[2]
            .parse()
            .map_err(|_| format!("invalid application version {value:?}"))?,
    ])
}

fn validate_asset_name(value: &str, extension: &str) -> Result<()> {
    if value.is_empty()
        || value.starts_with('.')
        || value.contains("..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        || !value.to_ascii_lowercase().ends_with(extension)
    {
        return Err(format!("invalid {extension} update filename {value:?}"));
    }
    Ok(())
}

fn parse_manifest(text: &str) -> Result<Manifest> {
    let mut version: Option<String> = None;
    let mut exe_file: Option<String> = None;
    let mut msi_file: Option<String> = None;
    let mut dmg_file: Option<String> = None;
    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.splitn(3, ':');
        let line_version = fields.next().unwrap_or_default().trim();
        let kind = fields.next().unwrap_or_default().trim();
        let filename = fields.next().unwrap_or_default().trim();
        if line_version.is_empty() || kind.is_empty() || filename.is_empty() {
            return Err(format!("invalid current-version line {}", index + 1));
        }
        parse_version(line_version)?;
        if let Some(expected) = &version {
            if expected != line_version {
                return Err("current-version entries do not use the same version".into());
            }
        } else {
            version = Some(line_version.to_string());
        }
        match kind {
            "exe" => {
                validate_asset_name(filename, ".exe")?;
                if exe_file.replace(filename.to_string()).is_some() {
                    return Err("current-version contains more than one exe entry".into());
                }
            }
            "msi" => {
                validate_asset_name(filename, ".msi")?;
                if msi_file.replace(filename.to_string()).is_some() {
                    return Err("current-version contains more than one msi entry".into());
                }
            }
            "dmg" => {
                validate_asset_name(filename, ".dmg")?;
                if dmg_file.replace(filename.to_string()).is_some() {
                    return Err("current-version contains more than one dmg entry".into());
                }
            }
            _ => return Err(format!("unsupported current-version asset type {kind:?}")),
        }
    }
    if exe_file.is_none() && msi_file.is_none() && dmg_file.is_none() {
        return Err("current-version has no update assets".into());
    }
    Ok(Manifest {
        version: version.ok_or_else(|| "current-version is empty".to_string())?,
        exe_file,
        msi_file,
        dmg_file,
    })
}

fn http_client(timeout: Duration) -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(timeout)
        .user_agent(format!("QuarterMaster-M/{APP_VERSION}"))
        .build()
        .map_err(|error| format!("cannot initialize the update client: {error}"))
}

fn fetch_manifest() -> Result<Manifest> {
    let response = http_client(Duration::from_secs(30))?
        .get(MANIFEST_URL)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("cannot retrieve current-version: {error}"))?;
    if response.content_length().unwrap_or(0) > MAX_MANIFEST_BYTES {
        return Err("current-version is unexpectedly large".into());
    }
    let text = response
        .text()
        .map_err(|error| format!("cannot read current-version: {error}"))?;
    if text.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("current-version is unexpectedly large".into());
    }
    parse_manifest(&text)
}

fn update_state(latest: &str) -> Result<&'static str> {
    Ok(
        match parse_version(latest)?.cmp(&parse_version(APP_VERSION)?) {
            std::cmp::Ordering::Greater => "available",
            std::cmp::Ordering::Equal => "current",
            std::cmp::Ordering::Less => "newer",
        },
    )
}

fn update_info(manifest: Manifest) -> Result<UpdateInfo> {
    Ok(UpdateInfo {
        current_version: APP_VERSION.into(),
        latest_version: manifest.version.clone(),
        state: update_state(&manifest.version)?.into(),
        platform: current_platform().into(),
        exe_file: manifest.exe_file,
        msi_file: manifest.msi_file,
        dmg_file: manifest.dmg_file,
    })
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "other"
    }
}

fn release_url(version: &str, filename: &str) -> String {
    format!("{RELEASE_DOWNLOAD_ROOT}/v{version}/{filename}")
}

fn download_file(url: &str, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| "update destination has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create update folder {}: {error}", parent.display()))?;
    let part = parent.join(format!(
        ".{}.{}.download",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("quartermaster-update"),
        std::process::id()
    ));
    let result = (|| -> Result<()> {
        let mut response = http_client(Duration::from_secs(15 * 60))?
            .get(url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| format!("cannot download update: {error}"))?;
        if response.content_length().unwrap_or(0) > MAX_UPDATE_BYTES {
            return Err("update download exceeds the allowed size".into());
        }
        let mut output = File::create(&part)
            .map_err(|error| format!("cannot create {}: {error}", part.display()))?;
        let copied = io::copy(&mut response, &mut output)
            .map_err(|error| format!("cannot write {}: {error}", part.display()))?;
        if copied > MAX_UPDATE_BYTES {
            return Err("update download exceeds the allowed size".into());
        }
        output
            .flush()
            .map_err(|error| format!("cannot flush {}: {error}", part.display()))?;
        output
            .sync_all()
            .map_err(|error| format!("cannot finalize {}: {error}", part.display()))?;
        if destination.exists() {
            fs::remove_file(destination).map_err(|error| {
                format!(
                    "cannot replace existing update {}: {error}",
                    destination.display()
                )
            })?;
        }
        fs::rename(&part, destination).map_err(|error| {
            format!(
                "cannot move the completed update to {}: {error}",
                destination.display()
            )
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&part);
    }
    result
}

fn require_newer_manifest() -> Result<Manifest> {
    let manifest = fetch_manifest()?;
    if update_state(&manifest.version)? != "available" {
        return Err(format!(
            "QuarterMaster/M {APP_VERSION} is already current (published version {}).",
            manifest.version
        ));
    }
    Ok(manifest)
}

fn download_portable() -> Result<UpdateDownload> {
    let manifest = require_newer_manifest()?;
    let current = std::env::current_exe()
        .map_err(|error| format!("cannot locate the running executable: {error}"))?;
    let directory = current
        .parent()
        .ok_or_else(|| "the running executable has no containing folder".to_string())?;
    let exe_file = manifest
        .exe_file
        .ok_or_else(|| "current-version has no Windows portable EXE entry".to_string())?;
    let destination = directory.join(&exe_file);
    if destination == current {
        return Err("the update filename would overwrite the running executable".into());
    }
    download_file(&release_url(&manifest.version, &exe_file), &destination)?;
    Ok(UpdateDownload {
        version: manifest.version,
        path: destination.to_string_lossy().into_owned(),
        launched: false,
    })
}

fn download_installer() -> Result<UpdateDownload> {
    let manifest = require_newer_manifest()?;
    let msi_file = manifest
        .msi_file
        .ok_or_else(|| "current-version has no Windows MSI entry".to_string())?;
    let directory = std::env::temp_dir().join("QuarterMaster-M").join("updates");
    let destination = directory.join(&msi_file);
    download_file(&release_url(&manifest.version, &msi_file), &destination)?;
    Command::new("msiexec.exe")
        .arg("/i")
        .arg(&destination)
        .spawn()
        .map_err(|error| format!("cannot launch Windows Installer: {error}"))?;
    Ok(UpdateDownload {
        version: manifest.version,
        path: destination.to_string_lossy().into_owned(),
        launched: true,
    })
}

fn download_macos_dmg() -> Result<UpdateDownload> {
    let manifest = require_newer_manifest()?;
    let dmg_file = manifest
        .dmg_file
        .ok_or_else(|| "current-version has no macOS DMG entry".to_string())?;
    let directory = std::env::temp_dir().join("QuarterMaster-M").join("updates");
    let destination = directory.join(&dmg_file);
    download_file(&release_url(&manifest.version, &dmg_file), &destination)?;
    if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&destination)
            .spawn()
            .map_err(|error| format!("cannot open macOS disk image: {error}"))?;
    } else {
        return Err("macOS updates are only supported on macOS".into());
    }
    Ok(UpdateDownload {
        version: manifest.version,
        path: destination.to_string_lossy().into_owned(),
        launched: true,
    })
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo> {
    tauri::async_runtime::spawn_blocking(|| fetch_manifest().and_then(update_info))
        .await
        .map_err(|error| format!("update check task failed: {error}"))?
}

#[tauri::command]
pub async fn download_portable_update() -> Result<UpdateDownload> {
    tauri::async_runtime::spawn_blocking(download_portable)
        .await
        .map_err(|error| format!("portable update task failed: {error}"))?
}

#[tauri::command]
pub async fn download_and_install_update() -> Result<UpdateDownload> {
    tauri::async_runtime::spawn_blocking(download_installer)
        .await
        .map_err(|error| format!("installer update task failed: {error}"))?
}

#[tauri::command]
pub async fn download_macos_update() -> Result<UpdateDownload> {
    tauri::async_runtime::spawn_blocking(download_macos_dmg)
        .await
        .map_err(|error| format!("macOS update task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_published_manifest_shape() {
        let manifest = parse_manifest(
            "1.2.3:exe:quartermaster-m-1.2.3.exe\n\
             1.2.3:msi:QuarterMaster-M_1.2.3_x64_en-US.msi\n",
        )
        .unwrap();
        assert_eq!(
            manifest,
            Manifest {
                version: "1.2.3".into(),
                exe_file: Some("quartermaster-m-1.2.3.exe".into()),
                msi_file: Some("QuarterMaster-M_1.2.3_x64_en-US.msi".into()),
                dmg_file: None,
            }
        );
    }

    #[test]
    fn parses_macos_manifest_entry() {
        let manifest = parse_manifest(
            "1.2.3:exe:quartermaster-m-1.2.3.exe\n\
             1.2.3:msi:QuarterMaster-M_1.2.3_x64_en-US.msi\n\
             1.2.3:dmg:QuarterMaster-M_1.2.3_universal.dmg\n",
        )
        .unwrap();
        assert_eq!(
            manifest.dmg_file,
            Some("QuarterMaster-M_1.2.3_universal.dmg".into())
        );
    }

    #[test]
    fn compares_semantic_versions_numerically() {
        assert!(parse_version("1.0.30").unwrap() > parse_version("1.0.29").unwrap());
        assert!(parse_version("1.10.0").unwrap() > parse_version("1.9.99").unwrap());
    }

    #[test]
    fn rejects_paths_and_mixed_manifest_versions() {
        assert!(parse_manifest(
            "1.2.3:exe:..\\evil.exe\n1.2.3:msi:QuarterMaster-M_1.2.3_x64_en-US.msi"
        )
        .is_err());
        assert!(parse_manifest(
            "1.2.3:exe:quartermaster-m-1.2.3.exe\n\
             1.2.4:msi:QuarterMaster-M_1.2.4_x64_en-US.msi"
        )
        .is_err());
    }
}
