use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        h1 { "404" }
        p { "Nothing at /{segments.join(\"/\")}" }
    }
}
