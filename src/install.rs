use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::color;

pub fn add_to_path() -> Result<()> {
    let exe = std::env::current_exe().context("Cannot find current exe path")?;
    let dir = exe.parent().unwrap().to_path_buf();

    #[cfg(windows)]
    add_to_path_windows(&dir, &exe)?;

    #[cfg(not(windows))]
    add_to_path_unix(&dir, &exe)?;

    Ok(())
}

#[cfg(windows)]
fn add_to_path_windows(dir: &PathBuf, _exe: &PathBuf) -> Result<()> {
    use std::process::Command;
    let dir_str = dir.to_string_lossy();

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command",
            "[Environment]::GetEnvironmentVariable('PATH','User')"])
        .output().context("Failed to read PATH")?;

    let current_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if current_path.split(';').any(|p| p.trim().eq_ignore_ascii_case(&dir_str)) {
        println!("{} Already in PATH: {}", color::green("✓"), color::spoak(&*dir_str));
        println!("  Open a new terminal and type {} to start.", color::spoak("spoak"));
        return Ok(());
    }

    let new_path = if current_path.is_empty() {
        dir_str.to_string()
    } else {
        format!("{};{}", current_path, dir_str)
    };

    let set_cmd = format!(
        "[Environment]::SetEnvironmentVariable('PATH', '{}', 'User')",
        new_path.replace('\'', "\\'")
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &set_cmd])
        .status().context("Failed to set PATH")?;

    if !status.success() { anyhow::bail!("Failed to update PATH"); }

    println!("{} Added to PATH: {}", color::green("✓"), color::spoak(&*dir_str));
    println!("  {} Open a new terminal and type {} to start.",
        color::dim("→"), color::spoak("spoak"));
    Ok(())
}

#[cfg(not(windows))]
fn add_to_path_unix(dir: &PathBuf, exe: &PathBuf) -> Result<()> {
    let bin_dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/bin");
    std::fs::create_dir_all(&bin_dir)?;
    let link = bin_dir.join("spoak");
    if link.exists() { std::fs::remove_file(&link)?; }
    std::os::unix::fs::symlink(exe, &link)?;
    println!("{} Symlink created: {}", color::green("✓"), color::spoak(&link.display().to_string()));
    println!("  Make sure {} is in your PATH.", color::spoak("~/.local/bin"));
    Ok(())
}
