use dioxus::prelude::*;

#[component]
pub fn Page(slug: String) -> Element {
    rsx! { h3 { "Post: {slug}" } }
}
