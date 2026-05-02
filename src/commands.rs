use anyhow::Result;
use crate::{api, color};

pub async fn ping(host: &str, port: u16) -> Result<()> {
    let s = api::mcping(host, port).await?;

    if s.online {
        println!("{} {}:{}", color::green("●"), color::bold(&s.host), port);

        if let Some(desc) = &s.description {
            let trimmed = desc.trim();
            if !trimmed.is_empty() {
                for line in trimmed.lines() {
                    println!("  {}", color::motd_to_ansi(line));
                }
            }
        }

        if let Some(ver) = &s.version {
            let sw = s.software.as_deref().unwrap_or("");
            if sw.is_empty() {
                println!("  Version  {}", color::spoak(ver));
            } else {
                println!("  Version  {} ({})", color::spoak(ver), color::yellow(sw));
            }
        }

        let online = s.players_online.unwrap_or(0);
        let max    = s.players_max.unwrap_or(0);
        println!("  Players  {}/{}", color::green(&online.to_string()), max);
        if !s.players.is_empty() {
            println!("           {}", color::dim(&s.players.join(", ")));
        }
        println!("  Latency  {}ms", color::green(&s.latency_ms.to_string()));
    } else {
        println!("{} {}:{} — offline ({}ms)",
            color::red("●"), s.host, port, s.latency_ms);
    }

    Ok(())
}

pub async fn player(username: &str) -> Result<()> {
    let p = api::player(username).await?;

    println!("{}", color::spoak(&p.username));
    println!("  UUID   {}", color::dim(&p.uuid_formatted));
    println!("  Model  {}", color::spoak_dim(&p.skin_model));
    if let Some(url) = &p.skin_url {
        println!("  Skin   {}", color::dim(url));
    }
    if let Some(url) = &p.cape_url {
        println!("  Cape   {}", color::dim(url));
    }

    Ok(())
}

pub async fn jars_versions() -> Result<()> {
    let v = api::jar_versions().await?;
    println!("{}", color::spoak("Available Minecraft versions:"));

    fn ver_color(idx: usize, s: String) -> color::Colored {
        match idx % 7 {
            0 => color::spoak(s),
            1 => color::sky(s),
            2 => color::mint(s),
            3 => color::orange(s),
            4 => color::rose(s),
            5 => color::cyan(s),
            _ => color::magenta(s),
        }
    }

    fn major(ver: &str) -> &str {
        let mut dots = 0;
        for (i, c) in ver.char_indices() {
            if c == '.' { dots += 1; if dots == 2 { return &ver[..i]; } }
        }
        ver
    }

    let mut color_idx = 0usize;
    let mut last_major = "";
    let cols = 6;
    let mut col = 0;

    for ver in &v.versions {
        let maj = major(ver);
        if maj != last_major {
            if last_major != "" { color_idx = (color_idx + 1) % 7; }
            last_major = maj;
        }

        if col == cols { println!(); col = 0; }
        print!("  {}", ver_color(color_idx, format!("{:<10}", ver)));
        col += 1;
    }
    println!();
    Ok(())
}

pub async fn jars_paper(version: &str, all: bool) -> Result<()> {
    let resp = api::paper_builds(version).await?;
    print_builds("Paper", version, &resp.builds, all);
    Ok(())
}

pub async fn jars_leaf(version: &str, all: bool) -> Result<()> {
    let resp = api::leaf_builds(version).await?;
    print_builds("Leaf", version, &resp.builds, all);
    Ok(())
}

fn print_builds(name: &str, version: &str, builds: &[api::JarBuild], all: bool) {
    if builds.is_empty() {
        println!("No builds found for {} {}", name, version);
        return;
    }

    if all {
        println!("{} {} builds:", color::bold(name), color::spoak(version));
        for b in builds.iter().rev().take(10) {
            println!("  build {}  {}  {}",
                color::yellow(&b.build),
                channel_str(&b.channel),
                color::dim(&b.download_url));
        }
        if builds.len() > 10 {
            println!("  {} and {} more", color::dim("..."), builds.len() - 10);
        }
    } else {
        let latest = builds.iter().rev()
            .find(|b| b.channel == "stable")
            .or_else(|| builds.iter().rev().find(|b| b.channel == "experimental"))
            .or_else(|| builds.last());

        if let Some(b) = latest {
            println!("{} {} — build {} {}",
                color::bold(name),
                color::spoak(version),
                color::yellow(&b.build),
                channel_str(&b.channel));
            println!("  {}", color::dim(&b.download_url));
        }
    }
}

fn channel_str(ch: &str) -> String {
    match ch {
        "stable"       => color::green(ch).to_string(),
        "experimental" => color::yellow(ch).to_string(),
        _              => ch.to_string(),
    }
}

pub fn coords_nether(x: f64, z: f64) -> Result<()> {
    println!("Overworld ({}, {}) {} Nether ({}, {})",
        color::spoak(&x.to_string()), color::spoak(&z.to_string()),
        color::dim("→"),
        color::yellow(&format!("{:.1}", x / 8.0)),
        color::yellow(&format!("{:.1}", z / 8.0)));
    Ok(())
}

pub fn coords_overworld(x: f64, z: f64) -> Result<()> {
    println!("Nether ({}, {}) {} Overworld ({}, {})",
        color::spoak(&x.to_string()), color::spoak(&z.to_string()),
        color::dim("→"),
        color::yellow(&format!("{:.1}", x * 8.0)),
        color::yellow(&format!("{:.1}", z * 8.0)));
    Ok(())
}

pub async fn seedmap_structures(seed: &str, x: i32, z: i32, radius: i32) -> Result<()> {
    println!("{} {} ...", color::dim("Searching structures near"), color::spoak(&format!("{}, {}", x, z)));
    let markers = api::seedmap_structures(seed, x, z, radius).await?;
    
    if markers.is_empty() {
        println!("  {}", color::yellow("No structures found in that radius."));
        return Ok(());
    }

    // Group by kind or just list them.
    for m in markers.iter().take(20) {
        let dist = (((m.x - x).pow(2) + (m.z - z).pow(2)) as f64).sqrt() as i32;
        println!("  {} {} ({}, {}) — {} blocks away",
            color::green("●"),
            color::bold(&m.label),
            color::spoak(&m.x.to_string()),
            color::spoak(&m.z.to_string()),
            color::dim(&dist.to_string()),
        );
    }

    if markers.len() > 20 {
        println!("  {} and {} more", color::dim("..."), markers.len() - 20);
    }
    Ok(())
}
