use dioxus::prelude::*;

#[component]
pub fn Page() -> Element {
    rsx! {
        h1 { "Home" }
        p { "This page lives in src/pages/page.rs." }
    }
}
