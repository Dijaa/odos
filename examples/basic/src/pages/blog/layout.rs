use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    rsx! {
        section {
            h2 { "Blog" }
            Outlet::<Route> {}
        }
    }
}
