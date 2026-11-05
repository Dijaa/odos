/// What a folder name inside `src/pages/` means in the URL.
#[derive(Debug, PartialEq)]
pub enum Segment {
    /// `about` → `/about`
    Static(String),
    /// `[slug]` → `/:slug`
    Dynamic(String),
    /// `[...rest]` → `/:..rest`
    CatchAll(String),
    /// `(admin)` → does not appear in the URL
    Group(String),
}

/// A name that can become a Rust struct field: `slug`, `user_id`, `_x`.
fn is_valid_ident(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false; // empty string
    };
    (first.is_ascii_alphabetic() || first == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn parse_segment(name: &str) -> Result<Segment, String> {
    if name.starts_with('[') && name.ends_with(']') {
        let inner = &name[1..name.len() - 1];

        let (param, is_catch_all) = match inner.strip_prefix("...") {
            Some(rest) => (rest, true),
            None => (inner, false),
        };

        if !is_valid_ident(param) {
            return Err(format!(
                "invalid parameter name in `{name}`: use only letters, numbers and `_`, \
                 starting with a letter (e.g. `[slug]`, `[user_id]`)"
            ));
        }

        if is_catch_all {
            Ok(Segment::CatchAll(param.into()))
        } else {
            Ok(Segment::Dynamic(param.into()))
        }
    } else if name.starts_with('(') && name.ends_with(')') {
        let group = &name[1..name.len() - 1];

        if !is_valid_ident(group) {
            return Err(format!(
                "invalid group name in `{name}`: use only letters, numbers and `_`"
            ));
        }

        Ok(Segment::Group(group.into()))
    } else {
        Ok(Segment::Static(name.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_segment() {
        assert_eq!(parse_segment("sobre"), Ok(Segment::Static("sobre".into())));
    }

    #[test]
    fn static_with_hyphen_is_ok() {
        assert_eq!(parse_segment("sobre-nos"), Ok(Segment::Static("sobre-nos".into())));
    }

    #[test]
    fn dynamic_segment() {
        assert_eq!(parse_segment("[slug]"), Ok(Segment::Dynamic("slug".into())));
    }

    #[test]
    fn catch_all_segment() {
        assert_eq!(parse_segment("[...rest]"), Ok(Segment::CatchAll("rest".into())));
    }

    #[test]
    fn group_segment() {
        assert_eq!(parse_segment("(admin)"), Ok(Segment::Group("admin".into())));
    }

    #[test]
    fn empty_param_is_error() {
        assert!(parse_segment("[]").is_err());
        assert!(parse_segment("[...]").is_err());
        assert!(parse_segment("()").is_err());
    }

    #[test]
    fn invalid_param_is_error() {
        assert!(parse_segment("[meu-slug]").is_err());
        assert!(parse_segment("[1id]").is_err());
    }
}