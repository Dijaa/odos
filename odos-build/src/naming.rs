use std::collections::HashMap;
use std::path::PathBuf;

use crate::scan::PageDir;
use crate::segment::Segment;

/// The URL pattern for a route, e.g. `/blog/:slug`. Groups are skipped.
pub fn route_path(segments: &[&Segment]) -> String {
    build_path(segments, true)
}

/// Like `route_path`, but without parameter names (`/blog/:`), so
/// `/blog/:a` and `/blog/:b` are detected as the same URL.
fn url_shape(segments: &[&Segment]) -> String {
    build_path(segments, false)
}

fn build_path(segments: &[&Segment], with_names: bool) -> String {
    let parts: Vec<String> = segments
        .iter()
        .filter_map(|s| match s {
            Segment::Static(n) => Some(n.clone()),
            Segment::Dynamic(n) => Some(format!(":{}", if with_names { n } else { "" })),
            Segment::CatchAll(n) => Some(format!(":..{}", if with_names { n } else { "" })),
            Segment::Group(_) => None,
        })
        .collect();
    format!("/{}", parts.join("/"))
}

/// PascalCase enum variant for a route: `blog/[slug]` → `BlogSlug`,
/// root → `Index`. Names starting with a digit get a `Page` prefix.
pub fn variant_name(segments: &[&Segment]) -> String {
    let mut name = String::new();
    for s in segments {
        let (Segment::Static(n) | Segment::Dynamic(n) | Segment::CatchAll(n)) = s else {
            continue; // groups are not part of the name
        };
        for word in n.split(|c: char| !c.is_alphanumeric()) {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                name.extend(first.to_uppercase());
                name.push_str(chars.as_str());
            }
        }
    }
    if name.is_empty() {
        return "Index".into();
    }
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert_str(0, "Page");
    }
    name
}

/// Fails if two pages end up with the same variant name or the same URL.
pub fn check_collisions(root: &PageDir) -> Result<(), String> {
    let mut names = HashMap::new();
    let mut urls = HashMap::new();
    if root.has_not_found {
        let file = root.path.join("not_found.rs");
        let catch_all = Segment::CatchAll(String::new());
        insert_unique(&mut names, "NotFound".into(), file.clone(), "variant")?;
        insert_unique(&mut urls, url_shape(&[&catch_all]), file, "URL")?;
    }
    walk(root, &mut Vec::new(), &mut names, &mut urls)
}

fn walk<'a>(
    dir: &'a PageDir,
    segments: &mut Vec<&'a Segment>,
    names: &mut HashMap<String, PathBuf>,
    urls: &mut HashMap<String, PathBuf>,
) -> Result<(), String> {
    if let Some(s) = &dir.segment {
        segments.push(s);
    }
    if dir.has_page {
        let file = dir.path.join("page.rs");
        insert_unique(names, variant_name(segments), file.clone(), "variant")?;
        insert_unique(urls, url_shape(segments), file, "URL")?;
    }
    for child in &dir.children {
        walk(child, segments, names, urls)?;
    }
    if dir.segment.is_some() {
        segments.pop();
    }
    Ok(())
}

fn insert_unique(
    seen: &mut HashMap<String, PathBuf>,
    key: String,
    file: PathBuf,
    what: &str,
) -> Result<(), String> {
    match seen.insert(key.clone(), file.clone()) {
        Some(other) => Err(format!(
            "{} and {} produce the same {what} `{key}`",
            other.display(),
            file.display()
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{scan, tests::tree};

    fn st(s: &str) -> Segment {
        Segment::Static(s.into())
    }
    fn dy(s: &str) -> Segment {
        Segment::Dynamic(s.into())
    }
    fn ca(s: &str) -> Segment {
        Segment::CatchAll(s.into())
    }
    fn gr(s: &str) -> Segment {
        Segment::Group(s.into())
    }

    // --- route_path ---

    #[test]
    fn path_root() {
        assert_eq!(route_path(&[]), "/");
    }

    #[test]
    fn path_mixed() {
        let (a, b, c, d) = (gr("admin"), st("blog"), dy("slug"), ca("rest"));
        assert_eq!(route_path(&[&a, &b, &c, &d]), "/blog/:slug/:..rest");
    }

    // --- variant_name ---

    #[test]
    fn name_root() {
        assert_eq!(variant_name(&[]), "Index");
        assert_eq!(variant_name(&[&gr("admin")]), "Index");
    }

    #[test]
    fn name_mixed() {
        let (a, b, c) = (st("blog"), dy("user_id"), ca("rest"));
        assert_eq!(variant_name(&[&a, &b, &c]), "BlogUserIdRest");
    }

    #[test]
    fn name_with_hyphen_and_group() {
        let (a, b) = (gr("admin"), st("sobre-nos"));
        assert_eq!(variant_name(&[&a, &b]), "SobreNos");
    }

    #[test]
    fn name_starting_with_digit() {
        assert_eq!(variant_name(&[&st("2024")]), "Page2024");
    }

    // --- check_collisions ---

    fn collisions(files: &[&str]) -> Result<(), String> {
        let tmp = tree(files);
        check_collisions(&scan(tmp.path()).unwrap())
    }

    #[test]
    fn no_collision() {
        assert_eq!(
            collisions(&["page.rs", "not_found.rs", "blog/page.rs", "blog/[slug]/page.rs"]),
            Ok(())
        );
    }

    #[test]
    fn same_variant_name() {
        let err = collisions(&["blog-post/page.rs", "blog_post/page.rs"]).unwrap_err();
        assert!(err.contains("variant `BlogPost`"), "{err}");
    }

    #[test]
    fn same_url_different_param_names() {
        let err = collisions(&["(a)/[x]/page.rs", "(b)/[y]/page.rs"]).unwrap_err();
        assert!(err.contains("URL"), "{err}");
    }

    #[test]
    fn not_found_name_is_reserved() {
        let err = collisions(&["not_found.rs", "not-found/page.rs"]).unwrap_err();
        assert!(err.contains("`NotFound`"), "{err}");
    }

    #[test]
    fn root_catch_all_clashes_with_not_found() {
        let err = collisions(&["not_found.rs", "[...rest]/page.rs"]).unwrap_err();
        assert!(err.contains("URL"), "{err}");
    }

    #[test]
    fn folders_without_page_are_ignored() {
        // `blog/` has no page.rs, so it does not clash with anything
        assert_eq!(collisions(&["blog/", "(x)/blog/page.rs"]), Ok(()));
    }
}
