use ait::update::{
    check_for_updates_with_base_url, latest_release_url, normalize_version,
    update_status_from_versions, update_status_message, GitHubRelease, UpdateStatus,
};
use httpmock::Method::GET;
use httpmock::MockServer;

#[test]
fn latest_release_url_points_to_github_latest() {
    assert_eq!(
        latest_release_url(),
        "https://github.com/aitsuki/ait/releases/latest"
    );
}

#[test]
fn normalize_version_strips_leading_v_and_whitespace() {
    assert_eq!(normalize_version(" v0.2.0 ").unwrap(), "0.2.0");
    assert_eq!(normalize_version("0.2.0").unwrap(), "0.2.0");
}

#[test]
fn update_status_reports_latest_when_versions_match() {
    let status = update_status_from_versions("v0.2.0", "v0.2.0").unwrap();
    assert_eq!(status, UpdateStatus::UpToDate);
}

#[test]
fn update_status_reports_update_available_when_remote_is_newer() {
    let status = update_status_from_versions("v0.2.0", "v0.2.1").unwrap();
    assert_eq!(
        status,
        UpdateStatus::UpdateAvailable {
            current_version: "v0.2.0".to_string(),
            latest_version: "v0.2.1".to_string(),
            release_url: latest_release_url().to_string(),
        }
    );
}

#[test]
fn update_status_compares_version_numbers_numerically() {
    let status = update_status_from_versions("v0.2.2", "v0.2.10").unwrap();

    assert_eq!(
        status,
        UpdateStatus::UpdateAvailable {
            current_version: "v0.2.2".to_string(),
            latest_version: "v0.2.10".to_string(),
            release_url: latest_release_url().to_string(),
        }
    );
}

#[test]
fn github_release_is_deserializable() {
    let release: GitHubRelease = serde_json::from_str(
        r#"{"tag_name":"v0.2.1","html_url":"https://example.test/releases/v0.2.1","name":"v0.2.1"}"#,
    )
    .unwrap();

    assert_eq!(release.tag_name, "v0.2.1");
    assert_eq!(release.html_url, "https://example.test/releases/v0.2.1");
    assert_eq!(release.name.as_deref(), Some("v0.2.1"));
}

#[test]
fn update_status_message_mentions_versions_and_release_url() {
    let status = UpdateStatus::UpdateAvailable {
        current_version: "v0.2.0".to_string(),
        latest_version: "v0.2.1".to_string(),
        release_url: latest_release_url().to_string(),
    };

    let message = update_status_message("v0.2.0", &status);

    assert!(message.contains("v0.2.0"));
    assert!(message.contains("v0.2.1"));
    assert!(message.contains(latest_release_url()));
}

#[tokio::test]
async fn check_for_updates_reports_update_available_from_github_api() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/aitsuki/ait/releases/latest")
            .header("accept", "application/vnd.github+json");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{"tag_name":"v0.2.1","html_url":"https://github.com/aitsuki/ait/releases/tag/v0.2.1","name":"v0.2.1"}"#,
            );
    });

    let status = check_for_updates_with_base_url("v0.2.0", &server.base_url())
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        status,
        UpdateStatus::UpdateAvailable {
            current_version: "v0.2.0".to_string(),
            latest_version: "v0.2.1".to_string(),
            release_url: latest_release_url().to_string(),
        }
    );
}

#[tokio::test]
async fn check_for_updates_reports_up_to_date_when_versions_match() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET).path("/repos/aitsuki/ait/releases/latest");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{"tag_name":"v0.2.0","html_url":"https://github.com/aitsuki/ait/releases/tag/v0.2.0","name":"v0.2.0"}"#,
            );
    });

    let status = check_for_updates_with_base_url("v0.2.0", &server.base_url())
        .await
        .unwrap();

    assert_eq!(status, UpdateStatus::UpToDate);
}
