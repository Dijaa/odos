use std::fs;
use std::path::Path;

use quote::ToTokens;

use crate::scan::PageDir;
use crate::segment::Segment;

/// One argument of `fn Page(...)`, e.g. `slug: String`.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    /// The type as source text, e.g. `"Vec < String >"`.
    pub ty: String,
}

/// Finds `pub fn Page` in the source of a `page.rs` and returns its arguments.
pub fn parse_page_params(source: &str) -> Result<Vec<Param>, String> {
    let file = syn::parse_file(source).map_err(|e| format!("could not parse file: {e}"))?;

    let page = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(f) if f.sig.ident == "Page" => Some(f),
            _ => None,
        })
        .ok_or("no `fn Page` found: every `page.rs` must export `#[component] pub fn Page`")?;

    if !matches!(page.vis, syn::Visibility::Public(_)) {
        return Err("`fn Page` must be `pub`".into());
    }

    page.sig
        .inputs
        .iter()
        .map(|arg| {
            let syn::FnArg::Typed(arg) = arg else {
                return Err("`fn Page` cannot take `self`".to_string());
            };
            let syn::Pat::Ident(pat) = &*arg.pat else {
                return Err("`fn Page` arguments must be plain names, like `slug: String`".into());
            };
            Ok(Param {
                name: pat.ident.to_string(),
                ty: arg.ty.to_token_stream().to_string(),
            })
        })
        .collect()
}

/// Reads `page.rs` from disk. Errors include the file path.
pub fn read_page_params(path: &Path) -> Result<Vec<Param>, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_page_params(&source).map_err(|e| format!("{}: {e}", path.display()))
}

/// Fills `page_params` on every folder with a `page.rs` and checks that
/// each `Page` takes exactly the parameters of its route.
pub fn read_signatures(root: &mut PageDir) -> Result<(), String> {
    walk(root, &mut Vec::new(), false)
}

/// `route_params`: names of the `[param]`s from the root down to `dir`.
fn walk(dir: &mut PageDir, route_params: &mut Vec<String>, under_catch_all: bool) -> Result<(), String> {
    let pushed = match &dir.segment {
        None | Some(Segment::Group(_)) => false,
        _ if under_catch_all => {
            return Err(format!(
                "{}: a catch-all `[...param]` must be the last segment of the route",
                dir.path.display()
            ));
        }
        Some(Segment::Static(_)) => false,
        Some(Segment::Dynamic(name) | Segment::CatchAll(name)) => {
            if route_params.contains(name) {
                return Err(format!(
                    "{}: parameter `{name}` is used twice in the same route",
                    dir.path.display()
                ));
            }
            route_params.push(name.clone());
            true
        }
    };
    let under_catch_all = under_catch_all || matches!(dir.segment, Some(Segment::CatchAll(_)));

    if dir.has_page {
        let path = dir.path.join("page.rs");
        let params = read_page_params(&path)?;

        let mut expected = route_params.clone();
        let mut found: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        expected.sort();
        found.sort();
        if expected != found {
            return Err(format!(
                "{}: `Page` takes {found:?} but the route has {expected:?}",
                path.display()
            ));
        }
        dir.page_params = params;
    }

    for child in &mut dir.children {
        walk(child, route_params, under_catch_all)?;
    }

    if pushed {
        route_params.pop();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::scan;
    use tempfile::TempDir;

    fn param(name: &str, ty: &str) -> Param {
        Param { name: name.into(), ty: ty.into() }
    }

    // --- parse_page_params ---

    #[test]
    fn no_params() {
        let src = "#[component] pub fn Page() -> Element { rsx! {} }";
        assert_eq!(parse_page_params(src), Ok(vec![]));
    }

    #[test]
    fn reads_names_and_types() {
        let src = "#[component] pub fn Page(slug: String, rest: Vec<String>) -> Element { todo!() }";
        assert_eq!(
            parse_page_params(src),
            Ok(vec![param("slug", "String"), param("rest", "Vec < String >")])
        );
    }

    #[test]
    fn ignores_other_items() {
        let src = "fn helper(x: u8) {} pub fn Page(id: u32) -> Element { todo!() }";
        assert_eq!(parse_page_params(src), Ok(vec![param("id", "u32")]));
    }

    #[test]
    fn missing_page_is_error() {
        assert!(parse_page_params("pub fn Home() {}").unwrap_err().contains("no `fn Page`"));
    }

    #[test]
    fn private_page_is_error() {
        assert!(parse_page_params("fn Page() {}").unwrap_err().contains("pub"));
    }

    #[test]
    fn pattern_argument_is_error() {
        assert!(parse_page_params("pub fn Page((a, b): (u8, u8)) {}").is_err());
    }

    #[test]
    fn syntax_error_is_error() {
        assert!(parse_page_params("pub fn Page(").is_err());
    }

    // --- read_signatures ---

    /// Writes the files into a temp folder and scans it.
    fn pages(files: &[(&str, &str)]) -> (TempDir, PageDir) {
        let tmp = TempDir::new().unwrap();
        for (path, source) in files {
            let full = tmp.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, source).unwrap();
        }
        let root = scan(tmp.path()).unwrap();
        (tmp, root) // keep `tmp` alive, or the folder is deleted
    }

    const NO_PARAMS: &str = "pub fn Page() -> Element { todo!() }";

    #[test]
    fn fills_params() {
        let (_tmp, mut root) = pages(&[
            ("page.rs", NO_PARAMS),
            ("blog/[slug]/page.rs", "pub fn Page(slug: String) -> Element { todo!() }"),
        ]);
        read_signatures(&mut root).unwrap();
        let slug = &root.children[0].children[0];
        assert_eq!(slug.page_params, vec![param("slug", "String")]);
    }

    #[test]
    fn groups_add_no_params() {
        let (_tmp, mut root) = pages(&[("(admin)/[id]/(x)/page.rs", "pub fn Page(id: u32) -> Element { todo!() }")]);
        assert_eq!(read_signatures(&mut root), Ok(()));
    }

    #[test]
    fn wrong_name_is_error() {
        let (_tmp, mut root) = pages(&[("[slug]/page.rs", "pub fn Page(id: String) -> Element { todo!() }")]);
        let err = read_signatures(&mut root).unwrap_err();
        assert!(err.contains("[slug]/page.rs") && err.contains("\"slug\""), "{err}");
    }

    #[test]
    fn missing_param_is_error() {
        let (_tmp, mut root) = pages(&[("[slug]/page.rs", NO_PARAMS)]);
        assert!(read_signatures(&mut root).is_err());
    }

    #[test]
    fn duplicate_param_is_error() {
        let (_tmp, mut root) = pages(&[("[id]/x/[id]/page.rs", NO_PARAMS)]);
        assert!(read_signatures(&mut root).unwrap_err().contains("twice"));
    }

    #[test]
    fn catch_all_must_be_last() {
        let (_tmp, mut root) = pages(&[("[...rest]/more/page.rs", NO_PARAMS)]);
        assert!(read_signatures(&mut root).unwrap_err().contains("last segment"));
    }

    #[test]
    fn sibling_params_do_not_leak() {
        // `[a]` must be popped before visiting `b`
        let (_tmp, mut root) = pages(&[
            ("[a]/page.rs", "pub fn Page(a: String) -> Element { todo!() }"),
            ("b/page.rs", NO_PARAMS),
        ]);
        assert_eq!(read_signatures(&mut root), Ok(()));
    }
}
