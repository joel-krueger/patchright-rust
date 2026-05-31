//! The dogfood deploy gate: serve the Trunk-built landing page and assert,
//! through a real browser driven by playwright-rs, that it works as advertised.
//! Because the site is a Leptos CSR/WASM app, these assertions also prove the
//! WASM bundle actually boots and renders (a static-HTML check could not).
//!
//! Run after building the site:
//!   (cd crates/site && trunk build --release)
//!   cargo test --manifest-path crates/site-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/site/dist` is absent so it never fails a run
//! that didn't build the site.

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use playwright_rs::{Playwright, expect};
use tower_http::services::ServeDir;

fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../site/dist")
}

/// Serve `dist/` on an ephemeral port; returns the bound address and the
/// server task handle.
async fn serve(dist: &PathBuf) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let app = Router::new().fallback_service(ServeDir::new(dist));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind site server");
    let addr = listener.local_addr().expect("local addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve site");
    });
    (addr, handle)
}

#[tokio::test]
async fn landing_page_boots_and_shows_hero() {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping dogfood test: {} not built — run `trunk build` in crates/site first",
            dist.display()
        );
        return;
    }

    let (addr, server) = serve(&dist).await;

    let pw = Playwright::launch().await.expect("launch playwright");
    let browser = pw.chromium().launch().await.expect("launch chromium");
    let page = browser.new_page().await.expect("new page");
    page.goto(&format!("http://{addr}"), None)
        .await
        .expect("navigate to site");

    // Auto-wait for Leptos to mount the hero — proves the WASM app booted.
    let hero = page.locator("#hero-title").await;
    expect(hero.clone())
        .to_be_visible()
        .await
        .expect("hero title should render");
    expect(hero)
        .to_contain_text("Playwright for Rust")
        .await
        .expect("hero title text");

    browser.close().await.ok();
    server.abort();
}
