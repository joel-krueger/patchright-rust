use leptos::prelude::*;

/// A tabbed code block: one header per language, showing the selected snippet.
/// The first interactive component, so the dogfood test exercises real
/// client-side reactivity (click a tab, the code switches).
#[component]
pub fn CodeTabs(
    /// `(label, highlighted_html)` pairs; the first is shown by default.
    tabs: Vec<(&'static str, &'static str)>,
) -> impl IntoView {
    let (active, set_active) = signal(0usize);
    let content_tabs = tabs.clone();

    let headers = tabs
        .iter()
        .enumerate()
        .map(|(i, (label, _))| {
            let label = *label;
            view! {
                <button
                    type="button"
                    data-lang=label
                    class=move || {
                        let base = "rounded-t-md px-3 py-1.5 text-xs font-semibold transition";
                        if active.get() == i {
                            format!("{base} bg-ink-800 text-rust-300")
                        } else {
                            format!("{base} text-rust-50/50 hover:text-rust-50/80")
                        }
                    }
                    on:click=move |_| set_active.set(i)
                >
                    {label}
                </button>
            }
        })
        .collect_view();

    let content = move || content_tabs[active.get()].1;

    view! {
        <div class="flex flex-col">
            <div role="tablist" class="flex gap-1 border-b border-rust-700/30">
                {headers}
            </div>
            <pre
                class="overflow-x-auto rounded-b-lg rounded-tr-lg border border-rust-700/40 bg-ink-800 p-4 text-sm leading-relaxed"
                inner_html=content
            ></pre>
        </div>
    }
}
