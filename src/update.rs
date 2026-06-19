use crate::error::{AppError, Result};
use reqwest::StatusCode;
use serde::Deserialize;

pub const GITHUB_LATEST_RELEASE_URL: &str = "https://github.com/aitsuki/ait/releases/latest";
pub const GITHUB_REPO_LATEST_API_URL: &str =
    "https://api.github.com/repos/aitsuki/ait/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable {
        current_version: String,
        latest_version: String,
        release_url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub name: Option<String>,
}

pub fn latest_release_url() -> &'static str {
    GITHUB_LATEST_RELEASE_URL
}

pub fn normalize_version(version: &str) -> Result<String> {
    let trimmed = version.trim();
    let normalized = trimmed.strip_prefix('v').unwrap_or(trimmed);
    if normalized.is_empty() {
        return Err(AppError::Config("版本号为空".to_string()));
    }
    Ok(normalized.to_string())
}

pub fn update_status_from_versions(current_version: &str, latest_version: &str) -> Result<UpdateStatus> {
    let current = parse_version(current_version)?;
    let latest = parse_version(latest_version)?;
    if current == latest {
        Ok(UpdateStatus::UpToDate)
    } else if latest > current {
        Ok(UpdateStatus::UpdateAvailable {
            current_version: current_version.trim().to_string(),
            latest_version: latest_version.trim().to_string(),
            release_url: latest_release_url().to_string(),
        })
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}

pub fn update_status_message(current_version: &str, status: &UpdateStatus) -> String {
    match status {
        UpdateStatus::UpToDate => format!("当前版本 {current_version} 已是最新版本。"),
        UpdateStatus::UpdateAvailable {
            latest_version,
            release_url,
            ..
        } => {
            format!("发现新版本 {latest_version}，当前版本 {current_version}。打开最新 Release：{release_url}")
        }
    }
}

pub async fn check_for_updates(current_version: &str) -> Result<UpdateStatus> {
    check_for_updates_with_base_url(current_version, "https://api.github.com").await
}

pub async fn check_for_updates_with_base_url(
    current_version: &str,
    base_url: &str,
) -> Result<UpdateStatus> {
    let client = reqwest::Client::builder()
        .user_agent("ait/0.1")
        .build()
        .map_err(|err| AppError::Network(err.to_string()))?;
    let latest_release = fetch_latest_release(&client, base_url).await?;
    update_status_from_versions(current_version, &latest_release.tag_name)
}

async fn fetch_latest_release(client: &reqwest::Client, base_url: &str) -> Result<GitHubRelease> {
    let url = format!("{}/repos/aitsuki/ait/releases/latest", base_url.trim_end_matches('/'));
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(reqwest::header::USER_AGENT, "ait/0.1")
        .send()
        .await
        .map_err(|err| AppError::Network(err.to_string()))?;

    let status = response.status();
    if status == StatusCode::FORBIDDEN {
        return Err(AppError::Network("GitHub Releases 访问被拒绝".to_string()));
    }
    if !status.is_success() {
        return Err(AppError::Network(format!("获取最新版本失败，状态码: {status}")));
    }

    response
        .json::<GitHubRelease>()
        .await
        .map_err(|err| AppError::Network(err.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VersionParts {
    major: u64,
    minor: u64,
    patch: u64,
}

fn parse_version(version: &str) -> Result<VersionParts> {
    let normalized = normalize_version(version)?;
    let mut parts = normalized.split('.');
    let major = parts
        .next()
        .ok_or_else(|| AppError::Config("版本号格式无效".to_string()))?
        .parse::<u64>()
        .map_err(|_| AppError::Config("版本号格式无效".to_string()))?;
    let minor = parts
        .next()
        .ok_or_else(|| AppError::Config("版本号格式无效".to_string()))?
        .parse::<u64>()
        .map_err(|_| AppError::Config("版本号格式无效".to_string()))?;
    let patch = parts
        .next()
        .ok_or_else(|| AppError::Config("版本号格式无效".to_string()))?
        .parse::<u64>()
        .map_err(|_| AppError::Config("版本号格式无效".to_string()))?;

    if parts.next().is_some() {
        return Err(AppError::Config("版本号格式无效".to_string()));
    }

    Ok(VersionParts {
        major,
        minor,
        patch,
    })
}
