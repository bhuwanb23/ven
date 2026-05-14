//! HTTP-mocked tests for OSV + EOL clients.
//!
//! These tests run a local `mockito` server and point the clients at it via
//! `VEN_OSV_BASE_URL` / `VEN_EOL_BASE_URL`. Each test isolates its own
//! SQLite cache via a per-test `VEN_STORAGE_PATH` so they don't leak.
//!
//! Cross-platform: pure Rust mocks; runs identically on Windows / macOS /
//! Linux in CI.
//!
//! ## Serialisation
//!
//! All three tests mutate process-wide env vars (`VEN_STORAGE_PATH`,
//! `VEN_OSV_BASE_URL`, `VEN_OSV_TTL_SECS`, `VEN_EOL_BASE_URL`). Cargo runs
//! `#[tokio::test]` cases concurrently within one process, so without a
//! shared lock test A's `set_var` is clobbered by test B's, and when test A
//! eventually constructs its client it reads the path test B wrote — whose
//! tempdir may already have been dropped, manifesting as
//! `unable to open database file`. A `Mutex` held for the full body of each
//! test forces them to run one-at-a-time within the same process.

use std::sync::{Mutex, MutexGuard};

use ven::core::endoflife::EndOfLifeClient;
use ven::core::osv::{OsvClient, OsvQuery};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the shared env lock. Recovers from poison so a panicking sibling
/// test doesn't permanently disable the remaining ones.
fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn isolate_storage(label: &str) -> tempfile::TempDir {
    let tmp = tempfile::Builder::new()
        .prefix(&format!("ven-mock-{}-", label))
        .tempdir()
        .unwrap();
    std::env::set_var("VEN_STORAGE_PATH", tmp.path());
    tmp
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn osv_returns_vulns_from_mocked_server() {
    let _env = lock_env();
    let _tmp = isolate_storage("osv");
    let mut server = mockito::Server::new_async().await;

    // querybatch returns one vuln id for our single query.
    let _m1 = server
        .mock("POST", "/querybatch")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results":[{"vulns":[{"id":"GHSA-test-1234"}]}]}"#)
        .create_async()
        .await;

    // /vulns/<id> returns enrichment with HIGH severity + summary + fix.
    let _m2 = server
        .mock("GET", "/vulns/GHSA-test-1234")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "GHSA-test-1234",
                "summary": "Mock vulnerability for testing",
                "aliases": ["CVE-9999-0001"],
                "severity": [{"type":"CVSS_V3","score":"8.1 CVSS:3.1/AV:N"}],
                "affected": [{"ranges":[{"events":[{"introduced":"0"},{"fixed":"4.18.3"}]}]}]
            }"#,
        )
        .create_async()
        .await;

    std::env::set_var("VEN_OSV_BASE_URL", server.url());

    let client = OsvClient::new().expect("OsvClient::new");
    let queries = vec![OsvQuery::new("npm", "express", "4.18.2")];
    let reports = client
        .query_packages(&queries)
        .await
        .expect("query_packages");

    assert_eq!(reports.len(), 1);
    let r = &reports[0];
    assert_eq!(r.package, "express");
    assert_eq!(r.version, "4.18.2");
    assert_eq!(r.vulns.len(), 1);

    let v = &r.vulns[0];
    assert_eq!(v.id, "GHSA-test-1234");
    assert_eq!(v.severity_label, "HIGH");
    assert_eq!(v.cves, vec!["CVE-9999-0001".to_string()]);
    assert_eq!(v.fixed_version.as_deref(), Some("4.18.3"));
    assert!(v.summary.as_deref().unwrap_or("").contains("Mock"));

    std::env::remove_var("VEN_OSV_BASE_URL");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn osv_serves_stale_on_network_failure_after_warm_cache() {
    let _env = lock_env();
    let _tmp = isolate_storage("osv-stale");
    // TTL=0 so every read counts as expired — forces the fallback path
    // through the network on every call.
    std::env::set_var("VEN_OSV_TTL_SECS", "0");

    let mut server = mockito::Server::new_async().await;

    // First call succeeds → writes a row to osv_cache.
    let _m1 = server
        .mock("POST", "/querybatch")
        .with_status(200)
        .with_body(r#"{"results":[{"vulns":[]}]}"#)
        .create_async()
        .await;

    std::env::set_var("VEN_OSV_BASE_URL", server.url());

    let client = OsvClient::new().unwrap();
    let queries = vec![OsvQuery::new("npm", "axios", "0.21.0")];
    let warm = client.query_packages(&queries).await.unwrap();
    assert_eq!(warm.len(), 1);
    drop(_m1);

    // Second call: server returns 500 (network "failure"). Client should
    // fall back to the (stale) cached entry — from_cache = true.
    let _m2 = server
        .mock("POST", "/querybatch")
        .with_status(500)
        .create_async()
        .await;

    let cold = client.query_packages(&queries).await.unwrap();
    assert_eq!(cold.len(), 1);
    assert_eq!(cold[0].package, "axios");
    assert!(cold[0].from_cache);

    std::env::remove_var("VEN_OSV_BASE_URL");
    std::env::remove_var("VEN_OSV_TTL_SECS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn eol_picks_matching_cycle_from_mocked_endpoint() {
    let _env = lock_env();
    let _tmp = isolate_storage("eol");
    let mut server = mockito::Server::new_async().await;

    let _m = server
        .mock("GET", "/nodejs.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[
                {"cycle":"20","eol":"2099-04-30","support":"2099-10-24","latest":"20.20.2","lts":false},
                {"cycle":"18","eol":"2025-04-30","support":"2024-10-18","latest":"18.20.0","lts":true}
            ]"#,
        )
        .create_async()
        .await;

    std::env::set_var("VEN_EOL_BASE_URL", server.url());

    let client = EndOfLifeClient::new().unwrap();

    // Active 20 → not EOL (cycle 20 has eol = 2099-04-30 in the future).
    let r20 = client.report("nodejs", "20.10.0").await.unwrap();
    assert_eq!(r20.matched_cycle.as_deref(), Some("20"));
    assert!(!r20.eol_passed);
    assert_eq!(r20.latest.as_deref(), Some("20.20.2"));

    // 18 → already past EOL (2025-04-30 < today, since today is 2026+).
    let r18 = client.report("nodejs", "18.16.0").await.unwrap();
    assert_eq!(r18.matched_cycle.as_deref(), Some("18"));
    assert!(r18.eol_passed);

    std::env::remove_var("VEN_EOL_BASE_URL");
}
