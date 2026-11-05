use dioxus::prelude::*;

/// `(admin)` is a group: it does not appear in the URL, so this is `/painel`.
#[component]
pub fn Page() -> Element {
    rsx! { h1 { "Dashboard" } }
}
