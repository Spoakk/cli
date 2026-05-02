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
const ASSET_NAME: &str = "spoak-backend.exe";
const USER_AGENT: &str = "spoak-cli/0.1.0";
pub const API_BASE: &str = "http://localhost:4000/api/v2";
const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn spoak_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".spoak")
}

pub fn backend_path() -> PathBuf {
    spoak_dir().join(ASSET_NAME)
}

fn version_file_path() -> PathBuf {
    spoak_dir().join("version.json")
}

#[derive(Serialize, Deserialize, Default)]
struct VersionCache {
    backend_tag: String,
    backend_sha256: String,
    cli_version: String,
    cli_latest_tag: String,
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
        if asset["name"].as_str() == Some(ASSET_NAME) {
            download_url = asset["browser_download_url"].as_str().map(|s| s.to_string());
            break;
        }
    }

    let download_url = download_url
        .ok_or_else(|| anyhow!("Asset '{}' not found in release {}", ASSET_NAME, tag))?;

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

    println!("{} backend {} ...", color::spoak("Downloading"), color::yellow(&info.tag));

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
            print!("\r  {:.1}/{:.1} MB  {}%",
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

    println!("{} Backend {} ready", color::green("✓"), color::yellow(&info.tag));
    Ok(())
}

pub async fn ensure_and_start(client: &reqwest::Client) -> Result<Child> {
    let cache = read_version_cache();

    if let Ok(cli_release) = fetch_release_tag(client, CLI_GITHUB_API).await {
        let latest = cli_release.trim_start_matches('v').to_string();
        let current = CLI_VERSION.trim_start_matches('v');
        if latest != current {
            println!("{} CLI update available: v{} → v{}",
                color::yellow("↑"), current, color::spoak(&latest));
            println!("  Run: {}",
                color::dim("iwr -useb https://github.com/Spoakk/cli/releases/latest/download/spoak.exe -OutFile spoak.exe"));
            if let Ok(()) = self_update(client, &cli_release).await {
                println!("{} CLI updated to v{} — please restart.", color::green("✓"), color::spoak(&latest));
                std::process::exit(0);
            }
        }
        write_version_cache(&VersionCache {
            cli_latest_tag: cli_release,
            ..read_version_cache()
        });
    }

    if !cache.cli_version.is_empty() && cache.cli_version != CLI_VERSION {
        println!("{} CLI updated: {} → {}",
            color::spoak("↑"), color::dim(&cache.cli_version), color::green(CLI_VERSION));
    }

    let info = fetch_latest_release(client).await
        .context("Failed to check for backend updates (GitHub)")?;

    let path = backend_path();
    let needs_download = !path.exists()
        || cache.backend_tag != info.tag
        || cache.cli_version != CLI_VERSION;

    if needs_download {
        if path.exists() && cache.backend_tag != info.tag {
            println!("{} Backend update: {} → {}",
                color::spoak("↑"), color::dim(&cache.backend_tag), color::yellow(&info.tag));
        }
        download_backend(client, &info).await?;
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

    #[cfg(windows)]
    let asset_name = "spoak.exe";
    #[cfg(not(windows))]
    let asset_name = "spoak";

    let download_url = format!(
        "https://github.com/Spoakk/cli/releases/download/{}/{}",
        tag, asset_name
    );

    println!("{} Downloading CLI {} ...", color::spoak("Updating"), color::yellow(tag));

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



