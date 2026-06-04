use anyhow::Result;
use crate::{api, color};

pub async fn ping(host: &str, port: u16) -> Result<()> {
    let s = api::mcping(host, port).await?;
    let title = format!("{} {} {}", color::gradient_text("Ping", (111.,81.,218.), (244.,114.,182.)), color::bold(&s.host), color::dim(&port.to_string()));
    let mut b = color::BentoBox::new(&title);
    b.set_width(70);

    if s.online {
        b.add(&format!("{} {}", color::dim("Status: "), color::green(&format!("● Online ({}ms)", s.latency_ms))));
        if let Some(ver) = &s.version {
            let sw = s.software.as_deref().unwrap_or("");
            if sw.is_empty() {
                b.add(&format!("{} {}", color::dim("Version:"), color::spoak(ver)));
            } else {
                b.add(&format!("{} {} {}", color::dim("Version:"), color::spoak(ver), color::yellow(&format!("({})", sw))));
            }
        }
        let online = s.players_online.unwrap_or(0);
        let max    = s.players_max.unwrap_or(0);
        b.add(&format!("{} {} / {}", color::dim("Players:"), color::green(&online.to_string()), max));
        if !s.players.is_empty() {
            b.add(&format!("         {}", color::dim(&s.players.join(", "))));
        }
        if let Some(desc) = &s.description {
            let trimmed = desc.trim();
            if !trimmed.is_empty() {
                b.empty_line();
                b.add(&color::dim("MOTD:").to_string());
                for line in trimmed.lines() {
                    b.add(&format!("  {}", color::motd_to_ansi(line)));
                }
            }
        }
    } else {
        b.add(&format!("{} {}", color::dim("Status: "), color::red(&format!("● Offline ({}ms)", s.latency_ms))));
    }
    b.draw();
    Ok(())
}

pub async fn player(username: &str) -> Result<()> {
    let p = api::player(username).await?;
    let title = format!("{} {}", color::gradient_text("Player", (111.,81.,218.), (80.,220.,160.)), color::bold(&p.username));
    let mut b = color::BentoBox::new(&title);
    b.set_width(65);

    b.add(&format!("{} {}", color::dim("UUID: "), color::spoak(&p.uuid_formatted)));
    b.add(&format!("{} {}", color::dim("Model:"), color::yellow(&p.skin_model)));
    if let Some(url) = &p.skin_url {
        b.add(&format!("{} {}", color::dim("Skin: "), color::dim(url)));
    }
    if let Some(url) = &p.cape_url {
        b.add(&format!("{} {}", color::dim("Cape: "), color::dim(url)));
    }

    b.draw();
    Ok(())
}

pub async fn jars_versions() -> Result<()> {
    let v = api::jar_versions().await?;
    let title = color::gradient_text("Versions", (80.,220.,160.), (80.,180.,255.));
    let mut b = color::BentoBox::new(&title);
    b.set_width(65);

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
    let cols = 5;
    let mut col = 0;
    let mut current_line = String::new();

    for ver in &v.versions {
        let maj = major(ver);
        if maj != last_major {
            if last_major != "" { color_idx = (color_idx + 1) % 7; }
            last_major = maj;
        }

        current_line.push_str(&format!("{} ", ver_color(color_idx, format!("{:<10}", ver))));
        col += 1;
        if col == cols {
            b.add(&current_line);
            current_line.clear();
            col = 0;
        }
    }
    if !current_line.is_empty() {
        b.add(&current_line);
    }

    b.draw();
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

pub async fn jars_purpur(version: &str, all: bool) -> Result<()> {
    let resp = api::purpur_builds(version).await?;
    print_builds("Purpur", version, &resp.builds, all);
    Ok(())
}

pub async fn jars_folia(version: &str, all: bool) -> Result<()> {
    let resp = api::folia_builds(version).await?;
    print_builds("Folia", version, &resp.builds, all);
    Ok(())
}

fn print_builds(name: &str, version: &str, builds: &[api::JarBuild], all: bool) {
    let title = format!("{} {}", color::gradient_text("Jars", (255.,160.,50.), (255.,100.,140.)), color::bold(name));
    let mut b = color::BentoBox::new(&title);
    b.set_width(75);

    if builds.is_empty() {
        b.add(&format!("No builds found for {} {}", name, version));
        b.draw();
        return;
    }

    if all {
        b.add(&format!("{} builds for {}", color::bold(name), color::spoak(version)));
        b.empty_line();
        for bd in builds.iter().rev().take(10) {
            b.add(&format!("build {} {} {}", 
                color::yellow(&bd.build),
                channel_str(&bd.channel),
                color::dim(&bd.download_url)
            ));
        }
        if builds.len() > 10 {
            b.add(&format!("{} and {} more", color::dim("..."), builds.len() - 10));
        }
    } else {
        let latest = builds.iter().rev()
            .find(|b| b.channel == "stable")
            .or_else(|| builds.iter().rev().find(|b| b.channel == "experimental"))
            .or_else(|| builds.last());

        if let Some(bd) = latest {
            b.add(&format!("Version: {}", color::spoak(version)));
            b.add(&format!("Build:   {} {}", color::yellow(&bd.build), channel_str(&bd.channel)));
            b.add(&format!("URL:     {}", color::dim(&bd.download_url)));
        }
    }
    b.draw();
}

fn channel_str(ch: &str) -> String {
    match ch {
        "stable"       => color::green(ch).to_string(),
        "experimental" => color::yellow(ch).to_string(),
        _              => ch.to_string(),
    }
}

pub fn coords_nether(x: f64, z: f64) -> Result<()> {
    let title = color::gradient_text("Coords", (80.,220.,160.), (244.,114.,182.));
    let mut b = color::BentoBox::new(&title);
    b.add(&format!("Overworld ({}, {}) {} Nether ({}, {})",
        color::spoak(&x.to_string()), color::spoak(&z.to_string()),
        color::dim("→"),
        color::yellow(&format!("{:.1}", x / 8.0)),
        color::yellow(&format!("{:.1}", z / 8.0))));
    b.draw();
    Ok(())
}

pub fn coords_overworld(x: f64, z: f64) -> Result<()> {
    let title = color::gradient_text("Coords", (80.,220.,160.), (244.,114.,182.));
    let mut b = color::BentoBox::new(&title);
    b.add(&format!("Nether ({}, {}) {} Overworld ({}, {})",
        color::spoak(&x.to_string()), color::spoak(&z.to_string()),
        color::dim("→"),
        color::yellow(&format!("{:.1}", x * 8.0)),
        color::yellow(&format!("{:.1}", z * 8.0))));
    b.draw();
    Ok(())
}

pub async fn seedmap_structures(seed: &str, x: i32, z: i32, radius: i32) -> Result<()> {
    let title = color::gradient_text("Structures", (244.,114.,182.), (111.,81.,218.));
    let mut b = color::BentoBox::new(&title);
    b.set_width(70);
    
    b.add(&format!("{} {}", color::dim("Searching near"), color::spoak(&format!("{}, {}", x, z))));
    let markers = api::seedmap_structures(seed, x, z, radius).await?;
    
    if markers.is_empty() {
        b.empty_line();
        b.add(&color::yellow("No structures found in that radius.").to_string());
        b.draw();
        return Ok(());
    }

    b.empty_line();
    for m in markers.iter().take(20) {
        let dist = (((m.x - x).pow(2) + (m.z - z).pow(2)) as f64).sqrt() as i32;
        b.add(&format!("{} {} ({}, {}) {} blocks away",
            color::green("●"),
            color::bold(&m.label),
            color::spoak(&m.x.to_string()),
            color::spoak(&m.z.to_string()),
            color::dim(&dist.to_string()),
        ));
    }

    if markers.len() > 20 {
        b.add(&format!("{} and {} more", color::dim("..."), markers.len() - 20));
    }
    b.draw();
    Ok(())
}
