use leptos::mount::mount_to_body;
use leptos::prelude::*;

fn main() {
    mount_to_body(App);
}

/// Root component. Walking skeleton: just the hero. Sections (install,
/// comparison, feature cards, footer) land in subsequent commits, each guarded
/// by the playwright-rs dogfood test in `crates/site-e2e`.
#[component]
fn App() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-ink-900 text-rust-50 flex flex-col items-center justify-center px-6 text-center">
            <h1 id="hero-title" class="text-5xl sm:text-6xl font-bold tracking-tight text-rust-500">
                "Playwright for Rust"
            </h1>
            <p id="hero-tagline" class="mt-4 max-w-2xl text-lg text-rust-100/80">
                "Cross-browser end-to-end testing for Rust — official-quality bindings for Microsoft Playwright."
            </p>
        </main>
    }
}
