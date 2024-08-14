// use serde::{Deserialize, Serialize};
use lazy_regex::regex;
use std::borrow::Cow;

/// Internal Path Representation.
///
/// They obey the following rules:
///
///  * Valid UTF-8
///  * Are path-separated with /
///  * Do not begin or end with /
///  * Do not contain . or .. segments
///  * Do not contain multiple / characters right next to each other
///
/// We can actually infallibly convert to these constraints as long as the input
/// we're working with is already UTF-8, which Rust strings are! It's worthwhile
/// to note that this is actually much pickier in certain ways than actual UNIX
/// paths, and you may find some imports fail when dealing with non-unicode
/// paths. The official dirtabase/ark policy for these situations is "we aren't
/// making stuff for your use case."
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct IPR(String);

impl AsRef<str> for IPR {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> PartialEq<T> for IPR
where
    T: ToIPR,
{
    fn eq(&self, other: &T) -> bool {
        self.eq(&other.to_ipr())
    }
}

pub trait ToIPR: AsRef<str> {
    fn to_ipr(&self) -> IPR {
        let converted = IPR::canonize(self.as_ref());
        IPR(converted.to_string())
    }
}

impl ToIPR for &str {}
impl ToIPR for &&str {}
impl ToIPR for String {}
impl ToIPR for &String {}

impl From<&str> for IPR {
    fn from(other: &str) -> IPR {
        other.to_ipr()
    }
}
impl From<&&str> for IPR {
    fn from(other: &&str) -> IPR {
        other.to_ipr()
    }
}
impl From<String> for IPR {
    fn from(other: String) -> IPR {
        other.to_ipr()
    }
}
impl From<&String> for IPR {
    fn from(other: &String) -> IPR {
        other.to_ipr()
    }
}

impl IPR {
    pub fn is_well_formed(src: &str) -> bool {
        // Quick and nasty way of avoiding regex cost.
        // Probably should have benched this before all the work.
        // I am not immune to premature optimization.
        enum SM {
            Strt,
            Slsh,
            Dot1,
            Dot2,
            Char,
        }

        let mut sm = SM::Strt;
        for c in src.chars() {
            // nc = next character
            sm = match (sm, c) {
                (SM::Strt, '/') => return false, // No leading slashes
                (SM::Strt, '.') => SM::Dot1,
                (SM::Strt, ___) => SM::Char,
                (SM::Slsh, '/') => return false, // Invalid: "//"
                (SM::Slsh, '.') => SM::Dot1,
                (SM::Slsh, ___) => SM::Char,
                (SM::Dot1, '/') => return false, // Invalid: "./"
                (SM::Dot1, '.') => SM::Dot2,
                (SM::Dot1, ___) => SM::Char,
                (SM::Dot2, '/') => return false, // Invalid: "../"
                (SM::Dot2, '.') => SM::Char,     // "..." is actually okay
                (SM::Dot2, ___) => SM::Char,
                (SM::Char, '/') => SM::Slsh,
                (SM::Char, '.') => SM::Char,
                (SM::Char, ___) => SM::Char,
            }
        }
        // What are valid final characters?
        match sm {
            SM::Strt => true,
            SM::Slsh => false,
            SM::Dot1 => false,
            SM::Dot2 => false,
            SM::Char => true,
        }
    }

    pub fn canonize<'a>(src: &'a str) -> Cow<'a, str> {
        if IPR::is_well_formed(&src) {
            Cow::Borrowed(src)
        } else {
            Cow::Owned(Self::force_canonize(&src))
        }
    }

    pub fn force_canonize(src: &str) -> String {
        let r = regex!("/");
        r.split(src)
            .filter(|s| *s != "" && *s != "." && *s != "..")
            .collect::<Vec<&str>>()
            .join("/")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_well_formed() {
        fn check(case: &str, expected: bool) {
            assert_eq!(IPR::is_well_formed(case), expected, "Failed on: {:?}", case);
        }
        check("", true);
        check("foo", true);
        check("foo/bar", true);
        check("/foo/bar", false);
        check("/foo/bar/", false);
        check("foo/bar////", false);
        check("////foo////bar", false);
        check(".", false);
        check("..", false);
        check("...", true);
        check("....", true);
        check(".....", true);
        check("foo.", true);
        check("a/./b", false);
        check("a/../b", false);
        check("a/.../b", true);
        check("a/..../b", true);
        check("a/...../b", true);
    }

    #[test]
    fn canonize() {
        fn check(case: &str, expected: &str) {
            assert_eq!(
                IPR::canonize(case),
                expected,
                "IPR::canonize failed on: {:?}",
                case
            );
            assert_eq!(
                IPR::force_canonize(case),
                expected,
                "IPR::force_canonize failed on: {:?}",
                case
            );
        }
        check("", "");
        check("foo", "foo");
        check("foo/bar", "foo/bar");
        check("/foo/bar", "foo/bar");
        check("/foo/bar/", "foo/bar");
        check("foo/bar////", "foo/bar");
        check("////foo////bar", "foo/bar");
        check(".", "");
        check("..", "");
        check("...", "...");
        check("....", "....");
        check(".....", ".....");
        check("foo.", "foo.");
        check("a/./b", "a/b");
        check("a/../b", "a/b");
        check("a/.../b", "a/.../b");
        check("a/..../b", "a/..../b");
        check("a/...../b", "a/...../b");
    }

    #[test]
    fn convert_str() {
        let original: &str = "/hello/world/";
        let converted: IPR = original.into();
        assert_eq!(converted.0, "hello/world".to_owned());
    }

    #[test]
    fn convert_string() {
        let original: String = "/hello/world/".to_owned();
        let converted: IPR = original.into();
        assert_eq!(converted.0, "hello/world".to_owned());
    }
}
