use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

impl Match {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexFlavor {
    Unprefixed,
    Pcre,
    Posix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSpec<'a> {
    pub flavor: RegexFlavor,
    pub display: Option<&'a str>,
    pub pattern: &'a str,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegexError {
    #[error("regex support is not enabled")]
    Disabled,
    #[error("pattern requires unsupported regex flavor: {0:?}")]
    UnsupportedFlavor(RegexFlavor),
    #[error("pattern requires UTF-8 input")]
    RequiresUtf8,
    #[error("regex compile failed: {0}")]
    Compile(String),
    #[error("regex match failed: {0}")]
    Match(String),
}

pub trait RegexEngine {
    type Regex;

    fn compile(spec: &PatternSpec<'_>) -> Result<Option<Self::Regex>, RegexError>;
    fn find_from(
        regex: &Self::Regex,
        haystack: &[u8],
        offset: usize,
    ) -> Result<Option<Match>, RegexError>;

    fn find_iter(regex: &Self::Regex, haystack: &[u8]) -> Result<Vec<Match>, RegexError> {
        let mut matches = Vec::new();
        let mut offset = 0;

        while offset < haystack.len() {
            let Some(found) = Self::find_from(regex, haystack, offset)? else {
                break;
            };

            if found.end > haystack.len() || found.start > found.end {
                return Err(RegexError::Match(
                    "regex engine returned invalid span".to_string(),
                ));
            }

            let is_empty = found.is_empty();
            offset = if is_empty { haystack.len() } else { found.end };
            matches.push(found);

            if is_empty {
                break;
            }
        }

        Ok(matches)
    }
}

pub fn parse_pattern_spec(raw: &str) -> PatternSpec<'_> {
    let (display, pattern) = parse_display_comment(raw);
    let (flavor, pattern) = if let Some(rest) = pattern.strip_prefix("PCRE:") {
        (RegexFlavor::Pcre, rest)
    } else if let Some(rest) = pattern.strip_prefix("POSIX:") {
        (RegexFlavor::Posix, rest)
    } else {
        (RegexFlavor::Unprefixed, pattern)
    };

    PatternSpec {
        flavor,
        display,
        pattern,
    }
}

fn parse_display_comment(raw: &str) -> (Option<&str>, &str) {
    let Some(rest) = raw.strip_prefix("(?#") else {
        return (None, raw);
    };

    let Some(comment_end) = rest.find(')') else {
        return (None, raw);
    };

    let display = &rest[..comment_end];
    let pattern = &rest[comment_end + 1..];
    (Some(display), pattern)
}

#[cfg(feature = "regex-bytes")]
pub mod bytes {
    use super::{Match, PatternSpec, RegexEngine, RegexError, RegexFlavor};

    pub struct BytesRegexEngine;

    impl RegexEngine for BytesRegexEngine {
        type Regex = regex::bytes::Regex;

        fn compile(spec: &PatternSpec<'_>) -> Result<Option<Self::Regex>, RegexError> {
            if spec.flavor == RegexFlavor::Pcre {
                return Ok(None);
            }

            regex::bytes::Regex::new(spec.pattern)
                .map(Some)
                .map_err(|err| RegexError::Compile(err.to_string()))
        }

        fn find_from(
            regex: &Self::Regex,
            haystack: &[u8],
            offset: usize,
        ) -> Result<Option<Match>, RegexError> {
            if offset > haystack.len() {
                return Ok(None);
            }

            Ok(regex.find(&haystack[offset..]).map(|found| Match {
                start: offset + found.start(),
                end: offset + found.end(),
            }))
        }
    }
}

#[cfg(feature = "fancy-regex")]
pub mod fancy {
    use super::{Match, PatternSpec, RegexEngine, RegexError};

    pub struct FancyRegexEngine;

    impl RegexEngine for FancyRegexEngine {
        type Regex = fancy_regex::Regex;

        fn compile(spec: &PatternSpec<'_>) -> Result<Option<Self::Regex>, RegexError> {
            fancy_regex::Regex::new(spec.pattern)
                .map(Some)
                .map_err(|err| RegexError::Compile(err.to_string()))
        }

        fn find_from(
            regex: &Self::Regex,
            haystack: &[u8],
            offset: usize,
        ) -> Result<Option<Match>, RegexError> {
            if offset > haystack.len() {
                return Ok(None);
            }

            let text =
                std::str::from_utf8(&haystack[offset..]).map_err(|_| RegexError::RequiresUtf8)?;
            regex
                .find(text)
                .map(|found| {
                    found.map(|found| Match {
                        start: offset + found.start(),
                        end: offset + found.end(),
                    })
                })
                .map_err(|err| RegexError::Match(err.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RegexEngine, RegexFlavor, parse_pattern_spec};

    #[cfg(feature = "regex-bytes")]
    use super::bytes::BytesRegexEngine;

    #[test]
    fn parses_chktex_display_comment_and_flavor() {
        let spec = parse_pattern_spec("(?#-2:Use booktabs)PCRE:\\\\hline");

        assert_eq!(spec.display, Some("-2:Use booktabs"));
        assert_eq!(spec.flavor, RegexFlavor::Pcre);
        assert_eq!(spec.pattern, "\\\\hline");
    }

    #[test]
    fn leaves_malformed_display_comment_as_pattern() {
        let spec = parse_pattern_spec("(?#missing close\\\\hline");

        assert_eq!(spec.display, None);
        assert_eq!(spec.flavor, RegexFlavor::Unprefixed);
        assert_eq!(spec.pattern, "(?#missing close\\\\hline");
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn bytes_engine_uses_chktex_offset_semantics() {
        let spec = parse_pattern_spec("^foo");
        let regex = BytesRegexEngine::compile(&spec).unwrap().unwrap();
        let haystack = b"bar foo";

        assert_eq!(
            BytesRegexEngine::find_from(&regex, haystack, 0).unwrap(),
            None
        );
        assert_eq!(
            BytesRegexEngine::find_from(&regex, haystack, 4)
                .unwrap()
                .unwrap(),
            super::Match { start: 4, end: 7 }
        );
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn bytes_engine_skips_pcre_prefixed_patterns() {
        let spec = parse_pattern_spec("PCRE:\\[(?!bad)");

        assert!(BytesRegexEngine::compile(&spec).unwrap().is_none());
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn bytes_engine_stops_iterating_on_empty_matches() {
        let spec = parse_pattern_spec("$");
        let regex = BytesRegexEngine::compile(&spec).unwrap().unwrap();

        assert_eq!(
            BytesRegexEngine::find_iter(&regex, b"abc").unwrap(),
            vec![super::Match { start: 3, end: 3 }]
        );
    }

    #[cfg(all(feature = "fancy-regex", feature = "regex-bytes"))]
    #[test]
    fn fancy_engine_compiles_default_pcre_only_pattern() {
        use super::fancy::FancyRegexEngine;

        let spec = parse_pattern_spec(r"PCRE:\[(?![^\]\[{}]*{(?![^\]\[{}]*}))[^\]]*\[");
        let regex = FancyRegexEngine::compile(&spec).unwrap().unwrap();
        let found = FancyRegexEngine::find_from(
            &regex,
            br"\begin{something}[\cite2[1231]{adadsd}]\end{something}",
            0,
        )
        .unwrap()
        .unwrap();

        assert_eq!(found.start, 17);
    }
}
