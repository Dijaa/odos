use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    rsx! {
        nav {
            Link { to: Route::Index {}, "Home" }
            " | "
            Link { to: Route::Sobre {}, "About" }
            " | "
            Link { to: Route::Blog {}, "Blog" }
            " | "
            Link { to: Route::Painel {}, "Dashboard" }
        }
        main { Outlet::<Route> {} }
    }
}
