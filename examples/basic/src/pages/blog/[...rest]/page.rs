use dioxus::prelude::*;

#[component]
pub fn Page(rest: Vec<String>) -> Element {
    rsx! { h3 { "Archive: {rest.join(\" / \")}" } }
}
