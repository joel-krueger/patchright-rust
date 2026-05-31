use leptos::prelude::*;

use super::FeatureCard;
use crate::snippets;

#[component]
pub fn Features() -> impl IntoView {
    view! {
        <section id="features" class="mx-auto max-w-5xl px-6 py-12">
            <h2 class="mb-6 text-2xl font-bold text-rust-300">"What you get"</h2>
            <div class="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
                <FeatureCard
                    id="feature-locators"
                    title="Auto-waiting locators"
                    blurb="Locators wait for elements to be actionable, so no sleeps and no flakes."
                    html=snippets::CARD_LOCATORS_RS
                />
                <FeatureCard
                    id="feature-assertions"
                    title="Auto-retrying assertions"
                    blurb="expect() retries until the DOM matches or the timeout elapses."
                    html=snippets::CARD_ASSERTIONS_RS
                />
                <FeatureCard
                    id="feature-cross-browser"
                    title="All three engines"
                    blurb="Chromium, Firefox, and WebKit run the same code across every browser."
                    html=snippets::CARD_CROSS_BROWSER_RS
                />
                <FeatureCard
                    id="feature-routing"
                    title="Network interception"
                    blurb="Mock, block, or inspect any request from Rust."
                    html=snippets::CARD_ROUTING_RS
                />
                <FeatureCard
                    id="feature-tracing"
                    title="Built-in observability"
                    blurb="Wire up tracing and every call emits structured spans."
                    html=snippets::CARD_TRACING_RS
                />
                <FeatureCard
                    id="feature-responsive"
                    title="Responsive testing"
                    blurb="Drive any viewport to test responsive layouts."
                    html=snippets::CARD_RESPONSIVE_RS
                />
            </div>
        </section>
    }
}
