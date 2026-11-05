use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Page() -> Element {
    rsx! {
        ul {
            li { Link { to: Route::BlogSlug { slug: "hello".into() }, "Hello" } }
            li { Link { to: Route::BlogRest { rest: vec!["2026".into(), "11".into()] }, "Archive 2026/11" } }
        }
    }
}
