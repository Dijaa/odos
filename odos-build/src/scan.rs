use std::fs;
use std::path::{Path, PathBuf};

use crate::segment::{Segment, parse_segment};
use crate::signature::Param;

/// A folder inside `src/pages/`.
#[derive(Debug)]
pub struct PageDir {
    /// Path to the folder.
    pub path: PathBuf,
    /// `None` for the root (`src/pages` itself).
    pub segment: Option<Segment>,
    pub has_page: bool,
    pub has_layout: bool,
    /// Only allowed at the root.
    pub has_not_found: bool,
    /// Arguments of `fn Page`, filled by `signature::read_signatures`.
    pub page_params: Vec<Param>,
    /// Subfolders, sorted by path so the generated code is stable.
    pub children: Vec<PageDir>,
}

/// Walks `root` (usually `src/pages/`) and builds the folder tree.
pub fn scan(root: &Path) -> Result<PageDir, String> {
    scan_dir(root, None)
}

fn scan_dir(dir: &Path, segment: Option<Segment>) -> Result<PageDir, String> {
    let is_root = segment.is_none();
    let io_err = |e: std::io::Error| format!("{}: {e}", dir.display());

    let mut node = PageDir {
        path: dir.to_path_buf(),
        segment,
        has_page: false,
        has_layout: false,
        has_not_found: false,
        page_params: Vec::new(),
        children: Vec::new(),
    };

    for entry in fs::read_dir(dir).map_err(io_err)? {
        let path = entry.map_err(io_err)?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return Err(format!("{}: file name is not valid UTF-8", path.display()));
        };

        if path.is_dir() {
            if name.starts_with('_') {
                continue; // private folder
            }
            let segment = parse_segment(name).map_err(|e| format!("{}: {e}", path.display()))?;
            node.children.push(scan_dir(&path, Some(segment))?);
            continue;
        }

        match name {
            "page.rs" => node.has_page = true,
            "layout.rs" => node.has_layout = true,
            "not_found.rs" if is_root => node.has_not_found = true,
            "not_found.rs" => {
                return Err(format!(
                    "{}: `not_found.rs` is only allowed directly in `src/pages/`",
                    path.display()
                ));
            }
            _ => {} // other files (helpers, components) are not routes
        }
    }

    node.children.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(node)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Creates a temp folder with the given empty files.
    /// A path ending in `/` creates an empty folder instead.
    pub(crate) fn tree(paths: &[&str]) -> TempDir {
        let tmp = TempDir::new().unwrap();
        for p in paths {
            let full = tmp.path().join(p);
            if p.ends_with('/') {
                fs::create_dir_all(&full).unwrap();
            } else {
                fs::create_dir_all(full.parent().unwrap()).unwrap();
                fs::write(&full, "").unwrap();
            }
        }
        tmp
    }

    #[test]
    fn root_flags() {
        let tmp = tree(&["page.rs", "layout.rs", "not_found.rs", "helpers.rs"]);
        let root = scan(tmp.path()).unwrap();
        assert!(root.segment.is_none());
        assert!(root.has_page && root.has_layout && root.has_not_found);
        assert!(root.children.is_empty());
    }

    #[test]
    fn nested_segments() {
        let tmp = tree(&["blog/page.rs", "blog/[slug]/page.rs", "(admin)/painel/page.rs"]);
        let root = scan(tmp.path()).unwrap();

        // sorted by name: "(admin)" < "blog"
        let [admin, blog] = &root.children[..] else {
            panic!("expected 2 children, got {:#?}", root.children);
        };
        assert_eq!(admin.segment, Some(Segment::Group("admin".into())));
        assert_eq!(admin.children[0].segment, Some(Segment::Static("painel".into())));
        assert!(admin.children[0].has_page);

        assert_eq!(blog.segment, Some(Segment::Static("blog".into())));
        assert!(blog.has_page);
        assert_eq!(blog.children[0].segment, Some(Segment::Dynamic("slug".into())));
    }

    #[test]
    fn children_are_sorted() {
        let tmp = tree(&["c/", "a/", "b/"]);
        let root = scan(tmp.path()).unwrap();
        let names: Vec<_> = root.children.iter().map(|c| c.path.file_name().unwrap()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    #[test]
    fn private_folders_are_ignored() {
        // even an invalid name inside `_components` must not cause an error
        let tmp = tree(&["_components/[bad-name]/page.rs", "page.rs"]);
        let root = scan(tmp.path()).unwrap();
        assert!(root.children.is_empty());
    }

    #[test]
    fn invalid_segment_error_has_path() {
        let tmp = tree(&["blog/[meu-slug]/page.rs"]);
        let err = scan(tmp.path()).unwrap_err();
        assert!(err.contains("blog/[meu-slug]"), "{err}");
    }

    #[test]
    fn not_found_outside_root_is_error() {
        let tmp = tree(&["blog/not_found.rs"]);
        let err = scan(tmp.path()).unwrap_err();
        assert!(err.contains("blog/not_found.rs"), "{err}");
    }

    #[test]
    fn missing_root_is_error() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("pages");
        let err = scan(&missing).unwrap_err();
        assert!(err.contains("pages"), "{err}");
    }
}
