use anyhow::Result;
use serde::Deserialize;
use crate::backend::API_BASE;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

#[derive(Deserialize)]
pub struct ServerStatus {
    pub online: bool,
    pub host: String,
    #[allow(dead_code)]
    pub port: u16,
    pub version: Option<String>,
    pub software: Option<String>,
    pub description: Option<String>,
    pub players_online: Option<u32>,
    pub players_max: Option<u32>,
    pub players: Vec<String>,
    pub latency_ms: u64,
}

pub async fn mcping(host: &str, port: u16) -> Result<ServerStatus> {
    let url = format!("{}/mcping?host={}&port={}", API_BASE, urlencoded(host), port);
    Ok(client().get(&url).send().await?.json().await?)
}

#[derive(Deserialize)]
pub struct PlayerProfile {
    pub uuid_formatted: String,
    pub username: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
    pub skin_model: String,
}

pub async fn player(username: &str) -> Result<PlayerProfile> {
    let url = format!("{}/player/{}", API_BASE, urlencoded(username));
    Ok(client().get(&url).send().await?.json().await?)
}

#[derive(Deserialize)]
pub struct Versions {
    pub versions: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct JarBuild {
    #[allow(dead_code)]
    pub version: String,
    pub build: String,
    pub channel: String,
    pub download_url: String,
}

#[derive(Deserialize)]
pub struct BuildsResponse {
    pub builds: Vec<JarBuild>,
}

pub async fn jar_versions() -> Result<Versions> {
    let url = format!("{}/serverjars/versions", API_BASE);
    Ok(client().get(&url).send().await?.json().await?)
}

pub async fn paper_builds(version: &str) -> Result<BuildsResponse> {
    let url = format!("{}/serverjars/paper/{}/builds", API_BASE, version);
    Ok(client().get(&url).send().await?.json().await?)
}

pub async fn leaf_builds(version: &str) -> Result<BuildsResponse> {
    let url = format!("{}/serverjars/leaf/{}/builds", API_BASE, version);
    Ok(client().get(&url).send().await?.json().await?)
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "%20").replace(':', "%3A")
}

