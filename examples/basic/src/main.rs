use dioxus::prelude::*;

odos::routes!();

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[cfg(test)]
mod tests {
    use super::Route;

    fn parse(url: &str) -> Route {
        url.parse().unwrap()
    }

    #[test]
    fn urls_match_the_right_page() {
        assert_eq!(parse("/"), Route::Index {});
        assert_eq!(parse("/sobre"), Route::Sobre {});
        assert_eq!(parse("/painel"), Route::Painel {});
        assert_eq!(parse("/blog"), Route::Blog {});
        assert_eq!(parse("/blog/hello"), Route::BlogSlug { slug: "hello".into() });
        assert_eq!(
            parse("/blog/a/b"),
            Route::BlogRest { rest: vec!["a".into(), "b".into()] }
        );
        assert_eq!(
            parse("/nope/x"),
            Route::NotFound { segments: vec!["nope".into(), "x".into()] }
        );
    }

    #[test]
    fn routes_render_back_to_urls() {
        assert_eq!(Route::BlogSlug { slug: "hello".into() }.to_string(), "/blog/hello");
    }
}
