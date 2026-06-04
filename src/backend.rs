use anyhow::{anyhow, Context, Result};
use crate::color;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Child;
use futures_util::StreamExt;

const GITHUB_API: &str = "https://api.github.com/repos/Spoakk/backend/releases/latest";
const CLI_GITHUB_API: &str = "https://api.github.com/repos/Spoakk/cli/releases/latest";
const USER_AGENT: &str = "spoak-cli/0.2.3";
pub const API_BASE: &str = "http://localhost:4000/api/v2";

fn remote_backend_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "spoak-backend-Windows.exe";
    #[cfg(target_os = "linux")]
    return "spoak-backend-Linux";
    #[cfg(target_os = "macos")]
    return "spoak-backend-macOS";
}

fn local_backend_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "spoak-backend.exe";
    #[cfg(not(target_os = "windows"))]
    return "spoak-backend";
}
const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn spoak_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".spoak")
}

pub fn backend_path() -> PathBuf {
    spoak_dir().join(local_backend_name())
}

fn version_file_path() -> PathBuf {
    spoak_dir().join("version.json")
}

#[derive(Serialize, Deserialize, Default)]
struct VersionCache {
    #[serde(default)]
    backend_tag: String,
    #[serde(default)]
    backend_sha256: String,
    #[serde(default)]
    cli_version: String,
    #[serde(default)]
    cli_latest_tag: String,
    #[serde(default)]
    last_check_time: Option<u64>,
}

fn read_version_cache() -> VersionCache {
    let path = version_file_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_version_cache(cache: &VersionCache) {
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(version_file_path(), json);
    }
}

struct ReleaseInfo {
    tag: String,
    download_url: String,
    sha256: Option<String>,
}

async fn fetch_latest_release(client: &reqwest::Client) -> Result<ReleaseInfo> {
    let release: serde_json::Value = client
        .get(GITHUB_API)
        .header("User-Agent", USER_AGENT)
        .send().await?
        .json().await?;

    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("No tag_name in release"))?
        .to_string();

    let assets = release["assets"].as_array()
        .ok_or_else(|| anyhow!("No assets in release"))?;

    let mut download_url = None;
    for asset in assets {
        if asset["name"].as_str() == Some(remote_backend_name()) {
            download_url = asset["browser_download_url"].as_str().map(|s| s.to_string());
            break;
        }
    }

    let download_url = download_url
        .ok_or_else(|| anyhow!("Asset '{}' not found in release {}", remote_backend_name(), tag))?;

    let sha256 = release["body"].as_str().and_then(|body| {
        body.lines()
            .find(|l| l.to_lowercase().starts_with("sha256:"))
            .map(|l| l["sha256:".len()..].trim().to_string())
    });

    Ok(ReleaseInfo { tag, download_url, sha256 })
}

async fn fetch_release_tag(client: &reqwest::Client, api_url: &str) -> Result<String> {
    let release: serde_json::Value = client
        .get(api_url)
        .header("User-Agent", USER_AGENT)
        .send().await?
        .json().await?;

    release["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("No tag_name in release"))
}

async fn download_backend(_client: &reqwest::Client, info: &ReleaseInfo) -> Result<()> {
    std::fs::create_dir_all(spoak_dir())?;

    let title = color::gradient_text("Updater", (111.,81.,218.), (244.,114.,182.));
    let mut b = color::BentoBox::new(&title);
    b.set_width(60);
    b.add(&format!("Downloading backend {} ...", color::yellow(&info.tag)));
    b.draw();

    let dl_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(USER_AGENT)
        .build()?;

    let resp = dl_client
        .get(&info.download_url)
        .header("Accept", "application/octet-stream")
        .send().await?;

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut bytes = Vec::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);
        if total > 0 {
            let pct = downloaded * 100 / total;
            let bars = (pct / 5) as usize;
            let empty = 20_usize.saturating_sub(bars);
            let bar_str = format!("{}{}", color::green(&"█".repeat(bars)), color::dim(&"░".repeat(empty)));
            print!("\r  {} {} {:.1}/{:.1} MB  {}%", 
                color::dim("Progress"),
                bar_str,
                downloaded as f64 / 1_000_000.0,
                total as f64 / 1_000_000.0,
                pct);
        }
    }
    println!();

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_hash = hex::encode(hasher.finalize());

    if let Some(expected) = &info.sha256 {
        if &actual_hash != expected {
            return Err(anyhow!(
                "SHA256 mismatch!\n  expected: {}\n  got:      {}",
                expected, actual_hash
            ));
        }
    }

    let path = backend_path();
    std::fs::write(&path, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }

    write_version_cache(&VersionCache {
        backend_tag: info.tag.clone(),
        backend_sha256: actual_hash,
        cli_version: CLI_VERSION.to_string(),
        cli_latest_tag: read_version_cache().cli_latest_tag,
    });

    println!("\n  {} Backend ready", color::green("✓"));
    Ok(())
}

pub async fn ensure_and_start(client: &reqwest::Client, force_check: bool) -> Result<Child> {
    let mut cache = read_version_cache();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let should_check = force_check || cache.last_check_time.map_or(true, |t| now - t > 3600);

    if should_check {
        if let Ok(cli_release) = fetch_release_tag(client, CLI_GITHUB_API).await {
            let latest = cli_release.trim_start_matches('v').to_string();
            let current = CLI_VERSION.trim_start_matches('v');
            
            if latest != current {
                if cache.cli_latest_tag != cli_release {
                    let title = color::gradient_text("Update Available", (255.,160.,50.), (255.,100.,140.));
                    let mut b = color::BentoBox::new(&title);
                    b.set_width(70);
                    b.add(&format!("CLI update: v{} {} v{}", current, color::dim("→"), color::green(&latest)));
                    b.empty_line();
                    b.add(&format!("Run the following command to update manually:"));
                    #[cfg(target_os = "windows")]
                    b.add(&color::dim("iwr -useb https://github.com/Spoakk/cli/releases/latest/download/spoak-cli-Windows.exe -OutFile spoak.exe").to_string());
                    #[cfg(not(target_os = "windows"))]
                    b.add(&color::dim("curl -Lo spoak https://github.com/Spoakk/cli/releases/latest/download/spoak-cli-Linux").to_string());
                    b.draw();
                    
                    cache.cli_latest_tag = cli_release.clone();
                    write_version_cache(&cache);

                    if let Ok(()) = self_update(client, &cli_release).await {
                        println!("\n  {} CLI updated to v{} — please restart.", color::green("✓"), color::green(&latest));
                        std::process::exit(0);
                    }
                }
            } else {
                cache.cli_latest_tag = cli_release;
            }
        }

        if !cache.cli_version.is_empty() && cache.cli_version != CLI_VERSION {
            let title = color::gradient_text("Update Complete", (80.,220.,160.), (111.,81.,218.));
            let mut b = color::BentoBox::new(&title);
            b.add(&format!("CLI updated: {} {} {}", color::dim(&cache.cli_version), color::dim("→"), color::green(CLI_VERSION)));
            b.draw();
            
            cache.cli_version = CLI_VERSION.to_string();
            write_version_cache(&cache);
        }

        if let Ok(info) = fetch_latest_release(client).await {
            let path = backend_path();
            let needs_download = !path.exists()
                || cache.backend_tag != info.tag
                || cache.cli_version != CLI_VERSION;

            if needs_download {
                if path.exists() && cache.backend_tag != info.tag {
                    let title = color::gradient_text("Backend Update", (80.,220.,160.), (111.,81.,218.));
                    let mut b = color::BentoBox::new(&title);
                    b.add(&format!("{} {} {}", color::dim(&cache.backend_tag), color::dim("→"), color::yellow(&info.tag)));
                    b.draw();
                }
                download_backend(client, &info).await?;
                cache = read_version_cache();
            }
        }

        cache.last_check_time = Some(now);
        write_version_cache(&cache);
    } else {
        if !backend_path().exists() {
            return Box::pin(ensure_and_start(client, true)).await;
        }
    }

    let child = tokio::process::Command::new(&path)
        .env("PORT", "4000")
        .env("ALLOWED_ORIGINS", "http://localhost")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start backend")?;

    wait_for_ready().await?;
    Ok(child)
}

async fn wait_for_ready() -> Result<()> {
    let c = reqwest::Client::new();
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if c.get(format!("{}/mcping?host=localhost", API_BASE))
            .send().await.is_ok()
        {
            return Ok(());
        }
    }
    Ok(())
}

async fn self_update(_client: &reqwest::Client, tag: &str) -> Result<()> {
    let current_exe = std::env::current_exe()?;

    #[cfg(target_os = "windows")]
    let remote_cli_name = "spoak-cli-Windows.exe";
    #[cfg(target_os = "linux")]
    let remote_cli_name = "spoak-cli-Linux";
    #[cfg(target_os = "macos")]
    let remote_cli_name = "spoak-cli-macOS";

    let download_url = format!(
        "https://github.com/Spoakk/cli/releases/download/{}/{}",
        tag, remote_cli_name
    );

    let title = color::gradient_text("Updating CLI", (111.,81.,218.), (244.,114.,182.));
    let mut b = color::BentoBox::new(&title);
    b.add(&format!("Downloading CLI {} ...", color::yellow(tag)));
    b.draw();

    let dl_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(USER_AGENT)
        .build()?;

    let resp = dl_client
        .get(&download_url)
        .header("Accept", "application/octet-stream")
        .send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Failed to download CLI update: {}", resp.status()));
    }

    let bytes = resp.bytes().await?;

    #[cfg(windows)]
    {
        let old_path = current_exe.with_extension("old");
        let _ = std::fs::remove_file(&old_path);
        std::fs::rename(&current_exe, &old_path)?;
    }

    std::fs::write(&current_exe, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}



