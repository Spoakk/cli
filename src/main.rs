mod backend;
mod api;
mod commands;
mod install;
mod color;

use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(
    name = "spoak",
    about = "Spoak CLI — Minecraft server tools in your terminal",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Ping {
        host: String,
        #[arg(short, long, default_value_t = 25565)]
        port: u16,
    },
    Player {
        username: String,
    },
    Jars {
        #[command(subcommand)]
        sub: JarsCommands,
    },
    Coords {
        #[command(subcommand)]
        sub: CoordsCommands,
    },
    Structures {
        seed: String,
        #[arg(short, long, default_value_t = 0)]
        x: i32,
        #[arg(short, long, default_value_t = 0)]
        z: i32,
        #[arg(short, long, default_value_t = 1024)]
        radius: i32,
    },
    Install,
}

#[derive(Subcommand)]
enum JarsCommands {
    Versions,
    Paper {
        version: String,
        #[arg(short, long)]
        all: bool,
    },
    Leaf {
        version: String,
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum CoordsCommands {
    Nether { x: f64, z: f64 },
    Overworld { x: f64, z: f64 },
}

#[tokio::main]
async fn main() {
    if std::env::args().len() == 1 && !is_terminal() {
        launch_interactive_terminal();
        return;
    }

    color::init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            run_interactive().await;
            return;
        }

        Some(Commands::Install) => {
            handle(install::add_to_path());
            return;
        }

        Some(Commands::Coords { sub }) => {
            let result = match sub {
                CoordsCommands::Nether { x, z } => commands::coords_nether(x, z),
                CoordsCommands::Overworld { x, z } => commands::coords_overworld(x, z),
            };
            handle(result);
            return;
        }

        Some(cmd) => {
            let http = make_client();
            let mut child = match backend::ensure_and_start(&http).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} {}", color::red("error:"), e);
                    std::process::exit(1);
                }
            };

            let result = match cmd {
                Commands::Ping { host, port } => commands::ping(&host, port).await,
                Commands::Player { username } => commands::player(&username).await,
                Commands::Jars { sub } => match sub {
                    JarsCommands::Versions => commands::jars_versions().await,
                    JarsCommands::Paper { version, all } => commands::jars_paper(&version, all).await,
                    JarsCommands::Leaf { version, all } => commands::jars_leaf(&version, all).await,
                },
                Commands::Structures { seed, x, z, radius } => commands::seedmap_structures(&seed, x, z, radius).await,
                _ => unreachable!(),
            };

            let _ = child.kill().await;
            handle(result);
        }
    }
}

async fn run_interactive() {
    print_banner();
    println!("Type a command or {} for help. {} to quit.\n", color::spoak("help"), color::dim("exit"));

    let http = make_client();
    let mut backend_child: Option<tokio::process::Child> = None;

    loop {
        print!("{} ", color::spoak_dim("spoak>"));
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() { break; }
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        if line == "exit" || line == "quit" { break; }

        let input = if line.starts_with("spoak ") { line[6..].to_string() } else { line.clone() };
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["help"] | ["--help"] | ["-h"] => print_help(),
            ["coords", "nether", x, z] => {
                if let (Ok(x), Ok(z)) = (x.parse(), z.parse()) {
                    handle(commands::coords_nether(x, z));
                } else { eprintln!("Invalid coordinates"); }
            }
            ["coords", "overworld", x, z] => {
                if let (Ok(x), Ok(z)) = (x.parse(), z.parse()) {
                    handle(commands::coords_overworld(x, z));
                } else { eprintln!("Invalid coordinates"); }
            }
            ["install"] => handle(install::add_to_path()),
            _ => {
                if backend_child.is_none() {
                    match backend::ensure_and_start(&http).await {
                        Ok(c) => backend_child = Some(c),
                        Err(e) => { eprintln!("{} {}", color::red("error:"), e); continue; }
                    }
                }
                let result = match parts.as_slice() {
                    ["ping", host] => commands::ping(host, 25565).await,
                    ["ping", host, port] => {
                        let p = port.parse().unwrap_or(25565);
                        commands::ping(host, p).await
                    }
                    ["player", username] => commands::player(username).await,
                    ["jars", "versions"] => commands::jars_versions().await,
                    ["jars", "paper", ver] => commands::jars_paper(ver, false).await,
                    ["jars", "paper", ver, "--all"] => commands::jars_paper(ver, true).await,
                    ["jars", "leaf", ver] => commands::jars_leaf(ver, false).await,
                    ["jars", "leaf", ver, "--all"] => commands::jars_leaf(ver, true).await,
                    ["structures", seed] => commands::seedmap_structures(seed, 0, 0, 1024).await,
                    ["structures", seed, x, z] => {
                        if let (Ok(x), Ok(z)) = (x.parse(), z.parse()) {
                            commands::seedmap_structures(seed, x, z, 1024).await
                        } else {
                            Err(anyhow::anyhow!("Invalid coordinates"))
                        }
                    }
                    _ => {
                        eprintln!("Unknown command. Type {} for help.", color::spoak("help"));
                        continue;
                    }
                };
                handle(result);
            }
        }
        println!();
    }

    if let Some(mut c) = backend_child {
        let _ = c.kill().await;
    }
}

fn print_banner() {
    println!("{}", color::spoak(r"
  ███████╗██████╗  ██████╗  █████╗ ██╗  ██╗
  ██╔════╝██╔══██╗██╔═══██╗██╔══██╗██║ ██╔╝
  ███████╗██████╔╝██║   ██║███████║█████╔╝ 
  ╚════██║██╔═══╝ ██║   ██║██╔══██║██╔═██╗ 
  ███████║██║     ╚██████╔╝██║  ██║██║  ██╗
  ╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝"));
    println!("  Minecraft server tools — v{}\n", color::dim(env!("CARGO_PKG_VERSION")));
}

fn print_help() {
    println!("{}", color::spoak("Commands:"));
    println!("  {}  <host> [port]     Ping a Minecraft server", color::spoak("ping"));
    println!("  {}  <username>        Look up a player profile", color::spoak("player"));
    println!("  {}  versions          List Minecraft versions", color::spoak("jars"));
    println!("  {}  paper <ver>       Latest Paper build", color::spoak("jars"));
    println!("  {}  leaf  <ver>       Latest Leaf build", color::spoak("jars"));
    println!("  {}  nether <x> <z>    Overworld → Nether coords", color::spoak("coords"));
    println!("  {}  overworld <x> <z> Nether → Overworld coords", color::spoak("coords"));
    println!("  {}  <seed> [x] [z]    Find structures in seed", color::spoak("structures"));
    println!("  {}                    Add spoak to PATH", color::spoak("install"));
    println!("  {}                    Quit", color::spoak("exit"));
}

fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("spoak-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap()
}

fn handle(r: anyhow::Result<()>) {
    if let Err(e) = r {
        eprintln!("{} {}", color::red("error:"), e);
    }
}

fn is_terminal() -> bool {
    #[cfg(windows)]
    {
        use std::ptr;
        extern "system" {
            fn GetConsoleWindow() -> *mut std::ffi::c_void;
        }
        unsafe { GetConsoleWindow() != ptr::null_mut() }
    }
    #[cfg(not(windows))]
    { true }
}

fn launch_interactive_terminal() {
    #[cfg(windows)]
    {
        let exe = std::env::current_exe().unwrap();
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", exe.to_str().unwrap()])
            .spawn();
    }
}

