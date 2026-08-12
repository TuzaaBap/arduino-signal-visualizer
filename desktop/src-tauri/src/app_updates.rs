use std::{sync::Mutex, time::Duration};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, ipc::Channel};
use tauri_plugin_updater::{Update, UpdaterExt};

const GITHUB_RELEASES_API: &str =
    "https://api.github.com/repos/TuzaaBap/arduino-signal-visualizer/releases?per_page=30";
const USER_AGENT: &str = "Arduino-Signal-Visualizer-Updater";
const UPDATE_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Default)]
pub struct UpdateManager(Mutex<Option<PendingUpdate>>);

#[derive(Clone)]
struct PendingUpdate {
    update: Update,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    version: String,
    current_version: String,
    notes: Option<String>,
    published_at: Option<String>,
    release_url: String,
    prerelease: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum DownloadEvent {
    Started { content_length: Option<u64> },
    Progress { chunk_length: usize },
    Installing,
    Complete,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    html_url: String,
    published_at: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseCandidate {
    app_version: Version,
    tag_version: Version,
    manifest_url: String,
    release_url: String,
    prerelease: bool,
    published_at: Option<String>,
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    manager: State<'_, UpdateManager>,
) -> Result<Option<UpdateMetadata>, String> {
    let current_version = app.package_info().version.clone();
    let releases = fetch_github_releases().await?;
    let Some(candidate) = select_release_candidate(&releases, &current_version) else {
        clear_pending(&manager)?;
        return Ok(None);
    };

    let endpoint: tauri::Url = candidate
        .manifest_url
        .parse()
        .map_err(|error| format!("GitHub returned an invalid update URL: {error}"))?;
    let updater = app
        .updater_builder()
        .timeout(UPDATE_TIMEOUT)
        .endpoints(vec![endpoint])
        .map_err(|error| format!("Could not configure the update endpoint: {error}"))?
        .build()
        .map_err(|error| format!("Could not initialize the updater: {error}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|error| format!("Could not validate the GitHub update manifest: {error}"))?
    else {
        clear_pending(&manager)?;
        return Ok(None);
    };

    let metadata = UpdateMetadata {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone().map(|notes| truncate_notes(&notes)),
        published_at: candidate.published_at.clone(),
        release_url: candidate.release_url.clone(),
        prerelease: candidate.prerelease,
    };
    let pending = PendingUpdate { update };
    *manager
        .0
        .lock()
        .map_err(|_| "The update state is unavailable".to_string())? = Some(pending);

    Ok(Some(metadata))
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    manager: State<'_, UpdateManager>,
    on_event: Channel<DownloadEvent>,
) -> Result<(), String> {
    let pending = manager
        .0
        .lock()
        .map_err(|_| "The update state is unavailable".to_string())?
        .clone()
        .ok_or_else(|| "There is no checked update ready to install".to_string())?;

    let mut started = false;
    let download_events = on_event.clone();
    let install_events = on_event.clone();
    pending
        .update
        .download_and_install(
            move |chunk_length, content_length| {
                if !started {
                    let _ = download_events.send(DownloadEvent::Started { content_length });
                    started = true;
                }
                let _ = download_events.send(DownloadEvent::Progress { chunk_length });
            },
            move || {
                let _ = install_events.send(DownloadEvent::Installing);
            },
        )
        .await
        .map_err(|error| format!("The signed update could not be installed: {error}"))?;

    let _ = on_event.send(DownloadEvent::Complete);
    app.restart();
}

#[tauri::command]
pub fn dismiss_update(manager: State<'_, UpdateManager>) -> Result<(), String> {
    clear_pending(&manager)
}

async fn fetch_github_releases() -> Result<Vec<GithubRelease>, String> {
    reqwest::Client::builder()
        .timeout(UPDATE_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| format!("Could not initialize the GitHub client: {error}"))?
        .get(GITHUB_RELEASES_API)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|error| format!("Could not reach GitHub Releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub Releases returned an error: {error}"))?
        .json::<Vec<GithubRelease>>()
        .await
        .map_err(|error| format!("GitHub returned an unreadable release list: {error}"))
}

fn select_release_candidate(
    releases: &[GithubRelease],
    current_version: &Version,
) -> Option<ReleaseCandidate> {
    releases
        .iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            let tag_version = parse_release_version(&release.tag_name)?;
            let app_version = installable_version(&tag_version);
            if app_version <= *current_version {
                return None;
            }
            let manifest = release
                .assets
                .iter()
                .find(|asset| asset.name.eq_ignore_ascii_case("latest.json"))?;
            Some(ReleaseCandidate {
                app_version,
                tag_version,
                manifest_url: manifest.browser_download_url.clone(),
                release_url: release.html_url.clone(),
                prerelease: release.prerelease,
                published_at: release.published_at.clone(),
            })
        })
        .max_by(|left, right| left.tag_version.cmp(&right.tag_version))
}

fn parse_release_version(tag: &str) -> Option<Version> {
    Version::parse(tag.trim().strip_prefix('v').unwrap_or(tag.trim())).ok()
}

fn installable_version(tag_version: &Version) -> Version {
    Version::new(tag_version.major, tag_version.minor, tag_version.patch)
}

fn truncate_notes(notes: &str) -> String {
    const MAX_CHARS: usize = 12_000;
    if notes.chars().count() <= MAX_CHARS {
        return notes.to_string();
    }
    let mut result: String = notes.chars().take(MAX_CHARS).collect();
    result.push_str("\n\nRelease notes truncated. Open the GitHub release to read the rest.");
    result
}

fn clear_pending(manager: &UpdateManager) -> Result<(), String> {
    *manager
        .0
        .lock()
        .map_err(|_| "The update state is unavailable".to_string())? = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, draft: bool, prerelease: bool, has_manifest: bool) -> GithubRelease {
        GithubRelease {
            tag_name: tag.to_string(),
            draft,
            prerelease,
            html_url: format!("https://github.test/releases/{tag}"),
            published_at: Some("2026-08-12T12:00:00Z".to_string()),
            assets: has_manifest
                .then(|| GithubAsset {
                    name: "latest.json".to_string(),
                    browser_download_url: format!("https://github.test/releases/{tag}/latest.json"),
                })
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn selects_newest_published_stable_release_with_an_update_manifest() {
        let releases = vec![
            release("v0.7.0-beta.1", false, true, true),
            release("v0.8.0", true, false, true),
            release("v0.9.0", false, false, false),
            release("v0.6.1", false, false, true),
        ];

        let selected = select_release_candidate(&releases, &Version::new(0, 6, 0)).unwrap();

        assert_eq!(selected.app_version, Version::new(0, 6, 1));
        assert_eq!(selected.tag_version, Version::new(0, 6, 1));
        assert!(!selected.prerelease);
        assert!(selected.manifest_url.ends_with("latest.json"));
    }

    #[test]
    fn stable_channel_ignores_published_prereleases() {
        let releases = vec![
            release("v0.8.0-beta.1", false, true, true),
            release("v0.7.0", false, false, true),
        ];

        let selected = select_release_candidate(&releases, &Version::new(0, 6, 0)).unwrap();

        assert_eq!(selected.app_version, Version::new(0, 7, 0));
        assert!(!selected.prerelease);
    }

    #[test]
    fn ignores_same_or_older_versions() {
        let releases = vec![
            release("v0.9.0-beta.1", false, true, true),
            release("v0.6.0", false, false, true),
            release("v0.5.1", false, false, true),
        ];

        assert!(select_release_candidate(&releases, &Version::new(0, 6, 0)).is_none());
    }

    #[test]
    fn parses_stable_and_prerelease_tags() {
        assert_eq!(parse_release_version("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(
            parse_release_version("v1.2.3-beta.4"),
            Version::parse("1.2.3-beta.4").ok()
        );
        assert_eq!(parse_release_version("not-a-release"), None);
        assert_eq!(
            installable_version(&Version::parse("1.2.3-beta.4").unwrap()),
            Version::new(1, 2, 3)
        );
    }
}
