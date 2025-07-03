//! Minimal markdown renderer.
//!
//! ...after the failures to port https://github.com/rambip/rust-web-markdown/tree/d7da996c05d6531ab2b18eaded8082ade75cc59e/leptos-markdown
//! to leptos >= 0.8.

use leptos::prelude::*;

use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, Options, Plugins};

#[component]
pub fn Markdown(#[prop(into)] text: Signal<String>) -> impl IntoView {
    view! { <div inner_html=move || md_to_html(&text.get()) /> }
}

fn md_to_html(text: &str) -> String {
    let options = Options::default();
    let mut plugins = Plugins::default();
    // Built-in themes:
    // - InspiredGitHub
    // - Solarized (dark)
    // - Solarized (light)
    // - base16-eighties.dark
    // - base16-mocha.dark
    // - base16-ocean.dark
    // - base16-ocean.light
    let adapter = SyntectAdapter::new(Some("InspiredGitHub"));
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    markdown_to_html_with_plugins(&text, &options, &plugins)
}
