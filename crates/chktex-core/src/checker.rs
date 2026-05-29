use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    diagnostic::{Diagnostic, DiagnosticKind},
    lexer::{Token, TokenKind, lex_line},
    regex_engine::{RegexEngine, parse_pattern_spec},
    resource::ResourceSet,
};

#[cfg(feature = "regex-bytes")]
use crate::regex_engine::bytes::BytesRegexEngine;
#[cfg(feature = "fancy-regex")]
use crate::regex_engine::fancy::FancyRegexEngine;

// Warning number constants
pub const WARNING_1: i32 = 1;
pub const WARNING_2: i32 = 2;
pub const WARNING_3: i32 = 3;
pub const WARNING_7: i32 = 7;
pub const WARNING_9: i32 = 9;
pub const WARNING_10: i32 = 10;
pub const WARNING_8: i32 = 8;
pub const WARNING_11: i32 = 11;
pub const WARNING_18: i32 = 18;
pub const WARNING_19: i32 = 19;
pub const WARNING_20: i32 = 20;
pub const WARNING_21: i32 = 21;
pub const WARNING_22: i32 = 22;
pub const WARNING_23: i32 = 23;
pub const WARNING_24: i32 = 24;
pub const WARNING_25: i32 = 25;
pub const WARNING_26: i32 = 26;
pub const WARNING_27: i32 = 27;
pub const WARNING_29: i32 = 29;
pub const WARNING_30: i32 = 30;
pub const WARNING_32: i32 = 32;
pub const WARNING_33: i32 = 33;
pub const WARNING_34: i32 = 34;
pub const WARNING_31: i32 = 31;
pub const WARNING_12: i32 = 12;
pub const WARNING_14: i32 = 14;
pub const WARNING_13: i32 = 13;
pub const WARNING_35: i32 = 35;
pub const WARNING_38: i32 = 38;
pub const WARNING_36: i32 = 36;
pub const WARNING_37: i32 = 37;
pub const WARNING_39: i32 = 39;
pub const WARNING_40: i32 = 40;
pub const WARNING_41: i32 = 41;
pub const WARNING_42: i32 = 42;
pub const WARNING_43: i32 = 43;
pub const WARNING_44: i32 = 44;
pub const WARNING_45: i32 = 45;
pub const WARNING_46: i32 = 46;
pub const WARNING_47: i32 = 47;
pub const WARNING_48: i32 = 48;
pub const WARNING_49: i32 = 49;

#[derive(Debug)]
pub struct CheckerConfig {
    pub silent_commands: BTreeSet<Vec<u8>>,
    #[cfg(feature = "regex-bytes")]
    pub silent_regex: Vec<regex::bytes::Regex>,
    pub user_warn: Vec<Vec<u8>>,
    pub user_warn_case_insensitive: Vec<Vec<u8>>,
    #[cfg(feature = "regex-bytes")]
    pub user_warn_regex: Vec<UserRegex>,
    pub math_commands: Vec<Vec<u8>>,
    pub text_commands: Vec<Vec<u8>>,
    pub math_envirs: Vec<Vec<u8>>,
    pub text_envirs: Vec<Vec<u8>>,
    pub verb_envirs: Vec<Vec<u8>>,
    pub math_roman: Vec<Vec<u8>>,
    pub primitives: Vec<Vec<u8>>,
    pub post_link: Vec<Vec<u8>>,
    pub not_pre_spaced: Vec<Vec<u8>>,
    pub linker: Vec<Vec<u8>>,
    pub no_char_next: Vec<(Vec<u8>, Vec<u8>)>,
    pub ij_accent: Vec<Vec<u8>>,
    pub italic_commands: Vec<Vec<u8>>,
    pub non_italic_commands: Vec<Vec<u8>>,
    pub hyph_dash: Vec<i64>,
    pub num_dash: Vec<i64>,
    pub word_dash: Vec<i64>,
    pub dash_excpt: Vec<Vec<u8>>,
    pub quote_style: bool, // true = Logical, false = Traditional
    pub abbrev: Vec<Vec<u8>>,
    pub abbrev_case: Vec<Vec<u8>>,
    pub wipe_arg: Vec<WipeArgEntry>,
    pub no_line_suppression: bool,
    pub header_errors: bool,
    pub wipe_verb: bool,
    pub cmd_space_style: CmdSpaceStyle,
    pub warning_kinds: HashMap<i32, DiagnosticKind>,
    pub warning_1_enabled: bool,
    pub user_warn_enabled: bool,
    pub user_warn_regex_enabled: bool,
    pub warning_2_enabled: bool,
    pub warning_3_enabled: bool,
    pub warning_4_enabled: bool,
    pub warning_5_enabled: bool,
    pub warning_6_enabled: bool,
    pub warning_7_enabled: bool,
    pub warning_28_enabled: bool,
    pub warning_8_enabled: bool,
    pub warning_9_enabled: bool,
    pub warning_10_enabled: bool,
    pub warning_11_enabled: bool,
    pub warning_12_enabled: bool,
    pub warning_13_enabled: bool,
    pub warning_14_enabled: bool,
    pub warning_15_enabled: bool,
    pub warning_16_enabled: bool,
    pub warning_17_enabled: bool,
    pub warning_18_enabled: bool,
    pub warning_19_enabled: bool,
    pub warning_21_enabled: bool,
    pub warning_22_enabled: bool,
    pub warning_23_enabled: bool,
    pub warning_24_enabled: bool,
    pub warning_25_enabled: bool,
    pub warning_26_enabled: bool,
    pub warning_27_enabled: bool,
    pub warning_29_enabled: bool,
    pub warning_30_enabled: bool,
    pub warning_31_enabled: bool,
    pub warning_32_enabled: bool,
    pub warning_33_enabled: bool,
    pub warning_34_enabled: bool,
    pub warning_35_enabled: bool,
    pub warning_36_enabled: bool,
    pub warning_37_enabled: bool,
    pub warning_38_enabled: bool,
    pub warning_39_enabled: bool,
    pub warning_40_enabled: bool,
    pub warning_41_enabled: bool,
    pub warning_42_enabled: bool,
    pub warning_43_enabled: bool,
    pub warning_45_enabled: bool,
    pub warning_46_enabled: bool,
    pub warning_47_enabled: bool,
    pub warning_48_enabled: bool,
    pub warning_49_enabled: bool,
}

#[cfg(feature = "regex-bytes")]
#[derive(Debug)]
pub struct UserRegex {
    pub display: Option<String>,
    pub regex: regex::bytes::Regex,
    #[cfg(feature = "fancy-regex")]
    pub fancy: Option<fancy_regex::Regex>,
}

#[derive(Debug)]
pub struct WipeArgEntry {
    pub command: Vec<u8>,
    pub spec: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdSpaceStyle {
    Ignore,
    InterWord,
    InterSentence,
    Both,
}

impl CmdSpaceStyle {
    fn allows_interword(self) -> bool {
        matches!(self, Self::InterWord | Self::Both)
    }

    fn allows_intersentence(self) -> bool {
        matches!(self, Self::InterSentence | Self::Both)
    }
}

impl Default for CheckerConfig {
    fn default() -> Self {
        Self {
            silent_commands: BTreeSet::new(),
            #[cfg(feature = "regex-bytes")]
            silent_regex: Vec::new(),
            user_warn: Vec::new(),
            user_warn_case_insensitive: Vec::new(),
            #[cfg(feature = "regex-bytes")]
            user_warn_regex: Vec::new(),
            math_commands: Vec::new(),
            text_commands: Vec::new(),
            math_envirs: vec![
                b"displaymath".to_vec(),
                b"math".to_vec(),
                b"eqnarray".to_vec(),
                b"array".to_vec(),
                b"equation".to_vec(),
                b"align".to_vec(),
                b"alignat".to_vec(),
                b"gather".to_vec(),
                b"flalign".to_vec(),
                b"multline".to_vec(),
                b"dmath".to_vec(),
                b"dgroup".to_vec(),
                b"darray".to_vec(),
            ],
            text_envirs: vec![b"dsuspend".to_vec()],
            verb_envirs: vec![
                b"verbatim".to_vec(),
                b"comment".to_vec(),
                b"listing".to_vec(),
                b"verbatimtab".to_vec(),
                b"rawhtml".to_vec(),
                b"errexam".to_vec(),
                b"picture".to_vec(),
                b"texdraw".to_vec(),
                b"filecontents".to_vec(),
                b"pgfpicture".to_vec(),
                b"tikzpicture".to_vec(),
                b"minted".to_vec(),
                b"lstlisting".to_vec(),
                b"IPA".to_vec(),
            ],
            math_roman: Vec::new(),
            primitives: Vec::new(),
            post_link: Vec::new(),
            not_pre_spaced: Vec::new(),
            linker: Vec::new(),
            no_char_next: Vec::new(),
            ij_accent: Vec::new(),
            wipe_arg: Vec::new(),
            italic_commands: vec![
                b"\\it".to_vec(),
                b"\\em".to_vec(),
                b"\\sl".to_vec(),
                b"\\itshape".to_vec(),
                b"\\slshape".to_vec(),
            ],
            non_italic_commands: vec![
                b"\\bf".to_vec(),
                b"\\rm".to_vec(),
                b"\\sf".to_vec(),
                b"\\tt".to_vec(),
                b"\\sc".to_vec(),
                b"\\upshape".to_vec(),
            ],
            hyph_dash: vec![1, 3],
            num_dash: vec![2],
            word_dash: vec![3],
            dash_excpt: Vec::new(),
            quote_style: true, // Logical by default
            abbrev: Vec::new(),
            abbrev_case: Vec::new(),
            no_line_suppression: false,
            header_errors: true,
            wipe_verb: true,
            cmd_space_style: CmdSpaceStyle::Ignore,
            warning_kinds: default_warning_kinds(),
            warning_1_enabled: true,
            user_warn_enabled: true,
            user_warn_regex_enabled: true,
            warning_2_enabled: true,
            warning_3_enabled: true,
            warning_4_enabled: true,
            warning_5_enabled: true,
            warning_6_enabled: true,
            warning_7_enabled: true,
            warning_28_enabled: true,
            warning_8_enabled: true,
            warning_9_enabled: true,
            warning_10_enabled: true,
            warning_11_enabled: true,
            warning_12_enabled: true,
            warning_13_enabled: true,
            warning_14_enabled: true,
            warning_15_enabled: true,
            warning_16_enabled: true,
            warning_17_enabled: true,
            warning_18_enabled: true,
            warning_19_enabled: false,
            warning_21_enabled: false,
            warning_22_enabled: false,
            warning_23_enabled: true,
            warning_24_enabled: true,
            warning_25_enabled: true,
            warning_26_enabled: true,
            warning_27_enabled: true,
            warning_29_enabled: true,
            warning_30_enabled: false,
            warning_31_enabled: true,
            warning_32_enabled: true,
            warning_33_enabled: true,
            warning_34_enabled: true,
            warning_35_enabled: true,
            warning_36_enabled: true,
            warning_37_enabled: true,
            warning_38_enabled: true,
            warning_39_enabled: true,
            warning_40_enabled: true,
            warning_41_enabled: false,
            warning_42_enabled: true,
            warning_43_enabled: true,
            warning_45_enabled: true,
            warning_46_enabled: false,
            warning_47_enabled: true,
            warning_48_enabled: true,
            warning_49_enabled: true,
        }
    }
}

impl CheckerConfig {
    pub fn set_warning_kind(&mut self, warning: i32, kind: DiagnosticKind) {
        self.warning_kinds.insert(warning, kind);
    }

    pub fn warning_kind(&self, warning: i32) -> DiagnosticKind {
        self.warning_kinds
            .get(&warning)
            .copied()
            .unwrap_or_else(|| default_warning_kind(warning))
    }

    pub fn warning_enabled(&self, warning: i32) -> bool {
        match warning {
            WARNING_1 => self.warning_1_enabled,
            WARNING_2 => self.warning_2_enabled,
            WARNING_3 => self.warning_3_enabled,
            4 => self.warning_4_enabled,
            5 => self.warning_5_enabled,
            6 => self.warning_6_enabled,
            WARNING_7 => self.warning_7_enabled,
            WARNING_8 => self.warning_8_enabled,
            WARNING_9 => self.warning_9_enabled,
            WARNING_10 => self.warning_10_enabled,
            WARNING_11 => self.warning_11_enabled,
            WARNING_12 => self.warning_12_enabled,
            WARNING_13 => self.warning_13_enabled,
            WARNING_14 => self.warning_14_enabled,
            15 => self.warning_15_enabled,
            16 => self.warning_16_enabled,
            17 => self.warning_17_enabled,
            WARNING_18 => self.warning_18_enabled,
            WARNING_19 => self.warning_19_enabled,
            WARNING_20 => self.user_warn_enabled,
            WARNING_21 => self.warning_21_enabled,
            WARNING_22 => self.warning_22_enabled,
            WARNING_23 => self.warning_23_enabled,
            WARNING_24 => self.warning_24_enabled,
            WARNING_25 => self.warning_25_enabled,
            WARNING_26 => self.warning_26_enabled,
            WARNING_27 => self.warning_27_enabled,
            28 => self.warning_28_enabled,
            WARNING_29 => self.warning_29_enabled,
            WARNING_30 => self.warning_30_enabled,
            WARNING_31 => self.warning_31_enabled,
            WARNING_32 => self.warning_32_enabled,
            WARNING_33 => self.warning_33_enabled,
            WARNING_34 => self.warning_34_enabled,
            WARNING_35 => self.warning_35_enabled,
            WARNING_36 => self.warning_36_enabled,
            WARNING_37 => self.warning_37_enabled,
            WARNING_38 => self.warning_38_enabled,
            WARNING_39 => self.warning_39_enabled,
            WARNING_40 => self.warning_40_enabled,
            WARNING_41 => self.warning_41_enabled,
            WARNING_42 => self.warning_42_enabled,
            WARNING_43 => self.warning_43_enabled,
            WARNING_44 => self.user_warn_regex_enabled,
            WARNING_45 => self.warning_45_enabled,
            WARNING_46 => self.warning_46_enabled,
            WARNING_47 => self.warning_47_enabled,
            WARNING_48 => self.warning_48_enabled,
            WARNING_49 => self.warning_49_enabled,
            _ => false,
        }
    }

    pub fn set_warning_enabled(&mut self, warning: i32, enabled: bool) {
        match warning {
            WARNING_1 => self.warning_1_enabled = enabled,
            WARNING_2 => self.warning_2_enabled = enabled,
            WARNING_3 => self.warning_3_enabled = enabled,
            4 => self.warning_4_enabled = enabled,
            5 => self.warning_5_enabled = enabled,
            6 => self.warning_6_enabled = enabled,
            WARNING_7 => self.warning_7_enabled = enabled,
            WARNING_8 => self.warning_8_enabled = enabled,
            WARNING_9 => self.warning_9_enabled = enabled,
            WARNING_10 => self.warning_10_enabled = enabled,
            WARNING_11 => self.warning_11_enabled = enabled,
            WARNING_12 => self.warning_12_enabled = enabled,
            WARNING_13 => self.warning_13_enabled = enabled,
            WARNING_14 => self.warning_14_enabled = enabled,
            15 => self.warning_15_enabled = enabled,
            16 => self.warning_16_enabled = enabled,
            17 => self.warning_17_enabled = enabled,
            WARNING_18 => self.warning_18_enabled = enabled,
            WARNING_19 => self.warning_19_enabled = enabled,
            WARNING_20 => self.user_warn_enabled = enabled,
            WARNING_21 => self.warning_21_enabled = enabled,
            WARNING_22 => self.warning_22_enabled = enabled,
            WARNING_23 => self.warning_23_enabled = enabled,
            WARNING_24 => self.warning_24_enabled = enabled,
            WARNING_25 => self.warning_25_enabled = enabled,
            WARNING_26 => self.warning_26_enabled = enabled,
            WARNING_27 => self.warning_27_enabled = enabled,
            28 => self.warning_28_enabled = enabled,
            WARNING_29 => self.warning_29_enabled = enabled,
            WARNING_30 => self.warning_30_enabled = enabled,
            WARNING_31 => self.warning_31_enabled = enabled,
            WARNING_32 => self.warning_32_enabled = enabled,
            WARNING_33 => self.warning_33_enabled = enabled,
            WARNING_34 => self.warning_34_enabled = enabled,
            WARNING_35 => self.warning_35_enabled = enabled,
            WARNING_36 => self.warning_36_enabled = enabled,
            WARNING_37 => self.warning_37_enabled = enabled,
            WARNING_38 => self.warning_38_enabled = enabled,
            WARNING_39 => self.warning_39_enabled = enabled,
            WARNING_40 => self.warning_40_enabled = enabled,
            WARNING_41 => self.warning_41_enabled = enabled,
            WARNING_42 => self.warning_42_enabled = enabled,
            WARNING_43 => self.warning_43_enabled = enabled,
            WARNING_44 => self.user_warn_regex_enabled = enabled,
            WARNING_45 => self.warning_45_enabled = enabled,
            WARNING_46 => self.warning_46_enabled = enabled,
            WARNING_47 => self.warning_47_enabled = enabled,
            WARNING_48 => self.warning_48_enabled = enabled,
            WARNING_49 => self.warning_49_enabled = enabled,
            _ => {}
        }
    }

    pub fn set_all_warnings_enabled(&mut self, enabled: bool) {
        for warning in 1..=49 {
            self.set_warning_enabled(warning, enabled);
        }
    }

    pub fn from_resources(resources: &ResourceSet) -> Self {
        let mut config = Self::default();
        if let Some(silent) = resources.get("Silent") {
            config.silent_commands = silent
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
            #[cfg(feature = "regex-bytes")]
            {
                config.silent_regex = silent
                    .case_insensitive_list
                    .iter()
                    .filter_map(|raw| {
                        let spec = parse_pattern_spec(raw);
                        BytesRegexEngine::compile(&spec).ok()?
                    })
                    .collect();
            }
        }
        if let Some(user_warn) = resources.get("UserWarn") {
            config.user_warn = user_warn
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
            config.user_warn_case_insensitive = user_warn
                .case_insensitive_list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        #[cfg(feature = "regex-bytes")]
        if let Some(user_warn_regex) = resources.get("UserWarnRegex") {
            #[cfg(feature = "fancy-regex")]
            {
                for raw in &user_warn_regex.list {
                    let spec = parse_pattern_spec(raw);
                    // Skip POSIX-only patterns when PCRE engine is available
                    // (upstream uses PCRE when available, POSIX patterns are redundant)
                    if spec.flavor == RegexFlavor::Posix {
                        continue;
                    }
                    if spec.flavor == RegexFlavor::Pcre {
                        if let Ok(Some(fancy)) = FancyRegexEngine::compile(&spec) {
                            config.user_warn_regex.push(UserRegex {
                                display: spec.display.map(ToOwned::to_owned),
                                regex: regex::bytes::Regex::new("").unwrap(),
                                fancy: Some(fancy),
                            });
                        }
                    } else if let Ok(Some(regex)) = BytesRegexEngine::compile(&spec) {
                        config.user_warn_regex.push(UserRegex {
                            display: spec.display.map(ToOwned::to_owned),
                            regex,
                            fancy: None,
                        });
                    }
                }
            }
            #[cfg(not(feature = "fancy-regex"))]
            {
                config.user_warn_regex = user_warn_regex
                    .list
                    .iter()
                    .filter_map(|raw| {
                        let spec = parse_pattern_spec(raw);
                        let regex = BytesRegexEngine::compile(&spec).ok()??;
                        Some(UserRegex {
                            display: spec.display.map(ToOwned::to_owned),
                            regex,
                            #[cfg(feature = "fancy-regex")]
                            fancy: None,
                        })
                    })
                    .collect();
            }
        }
        if let Some(math_cmd) = resources.get("MathCmd") {
            config.math_commands = math_cmd
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(text_cmd) = resources.get("TextCmd") {
            config.text_commands = text_cmd
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(math_env) = resources.get("MathEnvir") {
            config.math_envirs = math_env
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(text_env) = resources.get("TextEnvir") {
            config.text_envirs = text_env
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(verb_env) = resources.get("VerbEnvir") {
            config.verb_envirs = verb_env
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(math_roman) = resources.get("MathRoman") {
            config.math_roman = math_roman
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(prim) = resources.get("Primitives") {
            config.primitives = prim
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(post) = resources.get("PostLink") {
            config.post_link = post
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(nps) = resources.get("NotPreSpaced") {
            config.not_pre_spaced = nps
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(linker) = resources.get("Linker") {
            config.linker = linker
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(ncn) = resources.get("NoCharNext") {
            config.no_char_next = ncn
                .list
                .iter()
                .filter_map(|item| {
                    let (cmd, chars) = item.split_once(':')?;
                    Some((cmd.as_bytes().to_vec(), chars.as_bytes().to_vec()))
                })
                .collect();
        }
        if let Some(qs) = resources.get("QuoteStyle") {
            if let Some(v) = &qs.value {
                config.quote_style = v.eq_ignore_ascii_case("Logical");
            }
        }
        if let Some(cmd_space) = resources.get("CmdSpaceStyle") {
            if let Some(value) = &cmd_space.value {
                config.cmd_space_style = if value.eq_ignore_ascii_case("InterWord") {
                    CmdSpaceStyle::InterWord
                } else if value.eq_ignore_ascii_case("InterSentence") {
                    CmdSpaceStyle::InterSentence
                } else if value.eq_ignore_ascii_case("Both") {
                    CmdSpaceStyle::Both
                } else {
                    CmdSpaceStyle::Ignore
                };
            }
        }
        if let Some(ij) = resources.get("IJAccent") {
            config.ij_accent = ij
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(hd) = resources.get("HyphDash") {
            config.hyph_dash = hd
                .list
                .iter()
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
        }
        if let Some(nd) = resources.get("NumDash") {
            config.num_dash = nd
                .list
                .iter()
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
        }
        if let Some(wd) = resources.get("WordDash") {
            config.word_dash = wd
                .list
                .iter()
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
        }
        if let Some(de) = resources.get("DashExcpt") {
            config.dash_excpt = de
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(ab) = resources.get("Abbrev") {
            config.abbrev = ab
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
            config.abbrev_case = ab
                .case_insensitive_list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(it) = resources.get("Italic") {
            config.italic_commands = it
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(ni) = resources.get("NonItalic") {
            config.non_italic_commands = ni
                .list
                .iter()
                .map(|item| item.as_bytes().to_vec())
                .collect();
        }
        if let Some(wa) = resources.get("WipeArg") {
            config.wipe_arg = wa
                .list
                .iter()
                .filter_map(|item| {
                    let (cmd, spec) = item.split_once(':')?;
                    Some(WipeArgEntry {
                        command: cmd.as_bytes().to_vec(),
                        spec: spec.as_bytes().to_vec(),
                    })
                })
                .collect();
        }
        config
    }
}

pub const KNOWN_WARNINGS: &[i32] = &[
    WARNING_1, WARNING_2, WARNING_3, 4, 5, 6, WARNING_7, WARNING_8, WARNING_9, WARNING_10,
    WARNING_11, WARNING_12, WARNING_13, WARNING_14, 15, 16, 17, WARNING_18, WARNING_19, WARNING_20,
    WARNING_21, WARNING_22, WARNING_23, WARNING_24, WARNING_25, WARNING_26, WARNING_27, 28,
    WARNING_29, WARNING_30, WARNING_31, WARNING_32, WARNING_33, WARNING_34, WARNING_35, WARNING_36,
    WARNING_37, WARNING_38, WARNING_39, WARNING_40, WARNING_41, WARNING_42, WARNING_43, WARNING_44,
    WARNING_45, WARNING_46, WARNING_47, WARNING_48, WARNING_49,
];

fn default_warning_kinds() -> HashMap<i32, DiagnosticKind> {
    KNOWN_WARNINGS
        .iter()
        .map(|warning| (*warning, default_warning_kind(*warning)))
        .collect()
}

fn default_warning_kind(warning: i32) -> DiagnosticKind {
    match warning {
        WARNING_14 => DiagnosticKind::Error,
        WARNING_22 => DiagnosticKind::Message,
        _ => DiagnosticKind::Warning,
    }
}

// ====== Warning suppression ======

#[derive(Debug, Default)]
struct LineSuppressions {
    /// Warning numbers suppressed on this line. Contains -1 for "all".
    numbers: Vec<i64>,
}

fn parse_line_suppressions(line: &[u8]) -> LineSuppressions {
    // Look for % chktex N [M...] in comments.
    // Also handle `% chktex -1` for "suppress all on this line".
    let mut numbers = Vec::new();

    // Find all comment markers
    let line_str = std::str::from_utf8(line).unwrap_or("");
    for (mut i, _) in line_str.match_indices('%') {
        i += 1;
        let after_pct = &line_str[i..];

        let tokens: Vec<_> = after_pct.split_whitespace().collect();
        let mut idx = 0usize;
        while idx < tokens.len() {
            if tokens[idx] != "chktex" {
                idx += 1;
                continue;
            }
            idx += 1;
            while idx < tokens.len() {
                let token = tokens[idx];
                if token == "chktex" {
                    break;
                }
                if let Ok(n) = token.parse::<i64>() {
                    numbers.push(n);
                    idx += 1;
                } else {
                    break;
                }
            }
        }
    }

    LineSuppressions { numbers }
}

fn parse_file_suppressions(line: &[u8]) -> Option<HashSet<i64>> {
    // Look for % chktex-file N [M ...] or % CHKTEX-FILE N [M ...]
    let line_str = std::str::from_utf8(line).unwrap_or("");
    for (mut i, _) in line_str.match_indices('%') {
        i += 1;
        let after_pct = &line_str[i..];
        let keyword = after_pct.trim_start();

        let rest = keyword
            .strip_prefix("chktex-file")
            .or_else(|| keyword.strip_prefix("CHKTEX-FILE"))?;

        let mut numbers = HashSet::new();
        for token in rest.trim_start().split_whitespace() {
            if let Ok(n) = token.parse::<i64>() {
                numbers.insert(n);
            } else {
                break;
            }
        }
        if !numbers.is_empty() {
            return Some(numbers);
        }
    }
    None
}

fn is_suppressed(warning: i32, line: &LineSuppressions, file: &HashSet<i64>) -> bool {
    let w = i64::from(warning);
    line.numbers.contains(&-1) || line.numbers.contains(&w) || file.contains(&w)
}

// ====== Checker state ======

#[derive(Debug)]
struct CheckState {
    /// Stack of open environments (LaTeX)
    environment_stack: Vec<EnvFrame>,
    /// Stack of ConTeXt starts (\start... \stop...)
    context_stack: Vec<ContextFrame>,
    /// Stack of open brackets ({, [, ()
    bracket_stack: Vec<(u8, usize)>,
    /// Whether we're currently in math mode from $...$
    math_mode: bool,
    /// Whether display math is active (from $$...$$ or \[...\])
    display_math: bool,
    /// Whether the current math mode was opened by \( or \[.
    command_math_mode: bool,
    /// Whether we're inside a verbatim environment
    in_verbatim: bool,
    /// Per-file warning suppressions
    file_suppressions: HashSet<i64>,
    /// Italic correction state: 0=off, 1=on, 2=corrected
    italic_state: u8,
    /// Whether a space was seen (upstream SeenSpace) — affects leading space skip and W30
    seen_space: bool,
    /// Whether last line ended in a comment (upstream LastWasComment)
    last_was_comment: bool,
    /// Whether the previous line had TeX content before its comment marker.
    last_was_inline_comment: bool,
    /// Whether \frenchspacing is active.
    french_spacing: bool,
    /// Whether diagnostics are still in the pre-\begin{document} header.
    in_header: bool,
}

impl Default for CheckState {
    fn default() -> Self {
        Self {
            environment_stack: Vec::new(),
            context_stack: Vec::new(),
            bracket_stack: Vec::new(),
            math_mode: false,
            display_math: false,
            command_math_mode: false,
            in_verbatim: false,
            file_suppressions: HashSet::new(),
            italic_state: 0,
            seen_space: false,
            last_was_comment: false,
            last_was_inline_comment: false,
            french_spacing: false,
            in_header: true,
        }
    }
}

#[derive(Debug)]
struct EnvFrame {
    name: String,
    line: i64,
    column: usize,
    len: usize,
    source: Vec<u8>,
}

#[derive(Debug)]
struct ContextFrame {
    name: String,
    line: i64,
    column: usize,
    source: Vec<u8>,
}

// ====== Helper functions ======

fn command_name(token: &Token) -> Option<&[u8]> {
    match &token.kind {
        TokenKind::Command(c) => Some(c.as_slice()),
        _ => None,
    }
}

/// Check if a command matches any entry in a list of byte patterns.
fn matches_any(bytes: &[u8], list: &[Vec<u8>]) -> bool {
    list.iter().any(|item| item.as_slice() == bytes)
}

fn collapse_dash_runs(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut last_was_dash = false;
    for &b in bytes {
        if b == b'-' {
            if !last_was_dash {
                out.push(b);
            }
            last_was_dash = true;
        } else {
            out.push(b);
            last_was_dash = false;
        }
    }
    out
}

fn dash_run_len_at_collapsed(bytes: &[u8], collapsed_index: usize) -> Option<usize> {
    let mut collapsed_pos = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'-' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'-' {
                i += 1;
            }
            if collapsed_pos == collapsed_index {
                return Some(i - start);
            }
            collapsed_pos += 1;
        } else {
            if collapsed_pos == collapsed_index {
                return None;
            }
            collapsed_pos += 1;
            i += 1;
        }
    }
    None
}

/// Check if the environment name matches (with auto * variant).
fn env_matches(name: &str, envirs: &[Vec<u8>]) -> bool {
    let name_bytes = name.as_bytes();
    envirs.iter().any(|e| {
        e.as_slice() == name_bytes || {
            // * variant: env* matches env
            if let Some(base) = name.strip_suffix('*') {
                base.as_bytes() == e.as_slice()
            } else {
                false
            }
        }
    })
}

/// Check if a command is a Silent command (matching literal or regex).
fn is_silent_command(cmd: &[u8], config: &CheckerConfig) -> bool {
    if config.silent_commands.contains(cmd) {
        return true;
    }
    #[cfg(feature = "regex-bytes")]
    {
        for re in &config.silent_regex {
            if re.is_match(cmd) {
                return true;
            }
        }
    }
    false
}

// ====== Individual warning functions ======

fn warning_1_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
) -> Vec<Diagnostic> {
    if !config.warning_1_enabled {
        return Vec::new();
    }

    let mut in_math = initial_math;
    let mut diagnostics = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }

        let TokenKind::Command(command) = &token.kind else {
            continue;
        };

        if command.len() == 2
            || is_silent_command(command, config)
            || command.starts_with(br"\verb")
        {
            continue;
        }

        // Skip commands inside \verb content (preceded by \verb on this line)
        let inside_verb = config.wipe_verb
            && tokens[..index]
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Command(ref c) if c.starts_with(br"\verb")));
        if inside_verb {
            continue;
        }

        if let Some(space) = next_space_token(tokens, index) {
            diagnostics.push(Diagnostic::new(
                WARNING_1,
                DiagnosticKind::Warning,
                "",
                0,
                space.span.start,
                1,
                "Command terminated with space.",
                Vec::new(),
            ));
        }
    }
    diagnostics
}

/// Warning 2: Non-breaking space (`~') should have been used.
fn warning_2_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
) -> Vec<Diagnostic> {
    if !config.warning_2_enabled {
        return Vec::new();
    }
    let mut in_math = initial_math;
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }
        if !matches!(token.kind, TokenKind::Space) {
            continue;
        }
        // Look ahead for a Linker command (\ref, \vref, etc.)
        if let Some(next) = tokens.get(i + 1) {
            if let Some(cmd) = command_name(next) {
                if matches_any(cmd, &config.linker) && !has_tilde_before(tokens, i) {
                    diagnostics.push(Diagnostic::new(
                        WARNING_2,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        1,
                        "Non-breaking space (`~') should have been used.",
                        Vec::new(),
                    ));
                }
            }
        }
    }
    diagnostics.sort_by_key(|diag| (diag.line == 0, diag.line, diag.column));
    diagnostics
}

fn has_tilde_before(tokens: &[Token], space_index: usize) -> bool {
    // Look at tokens before the space for a ~ Punctuation token
    if space_index == 0 {
        return false;
    }
    if let Some(prev) = tokens.get(space_index - 1) {
        matches!(prev.kind, TokenKind::Punctuation(b'~'))
    } else {
        false
    }
}

/// Warning 3: You should enclose the previous parenthesis with `{}'.
fn warning_3_check(line: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_3_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    let mut in_math = false;
    for (pos, &b) in line.iter().enumerate() {
        if b == b'$' {
            in_math = !in_math;
            continue;
        }
        if !in_math || !matches!(b, b')' | b']') || line.get(pos + 1) != Some(&b'^') {
            continue;
        }
        diagnostics.push(Diagnostic::new(
            WARNING_3,
            DiagnosticKind::Message,
            "",
            0,
            pos,
            1,
            "You should enclose the previous parenthesis with `{}'.",
            Vec::new(),
        ));
    }
    diagnostics
}

/// Check raw bytes for W38 (quote style) — punct inside quotes in Logical style, or outside in Traditional.
fn warning_38_line_check(line: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if !config.warning_38_enabled {
        return diagnostics;
    }
    // In Logical style (default): comma/period before '' should be AFTER it
    // Pattern: ,'' or .'' → W38 "in front of"
    if config.quote_style {
        // In Logical style: punct before '' should be AFTER it
        for (pos, &b) in line.iter().enumerate() {
            if (b == b',' || b == b'.')
                && pos + 2 < line.len()
                && line[pos + 1] == b'\''
                && line[pos + 2] == b'\''
            {
                diagnostics.push(Diagnostic::new(
                    WARNING_38,
                    DiagnosticKind::Message,
                    "",
                    0,
                    pos,
                    1,
                    "You should not use punctuation in front of quotes.",
                    Vec::new(),
                ));
            }
        }
    }
    diagnostics
}

/// Check raw bytes for W34 (mixed quotes) — adjacent mixed quote chars.
fn warning_34_line_check(line: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if !config.warning_34_enabled {
        return diagnostics;
    }
    for (pos, &b) in line.iter().enumerate() {
        if pos + 2 < line.len()
            && ((line[pos] == b'`' && line[pos + 1] == b'"' && line[pos + 2] == b'`')
                || (line[pos] == b'\'' && line[pos + 1] == b'`' && line[pos + 2] == b'\''))
        {
            diagnostics.push(Diagnostic::new(
                WARNING_34,
                DiagnosticKind::Message,
                "",
                0,
                pos,
                3,
                "Don't mix quotes.",
                Vec::new(),
            ));
            continue;
        }
        if pos > 0
            && ((line[pos - 1] == b'`'
                && line[pos] == b'"'
                && pos + 1 < line.len()
                && line[pos + 1] == b'`')
                || (line[pos - 1] == b'\''
                    && line[pos] == b'`'
                    && pos + 1 < line.len()
                    && line[pos + 1] == b'\''))
        {
            continue;
        }
        // Look for ' followed by ` (or vice versa) adjacent
        if b == b'\'' && pos + 1 < line.len() && line[pos + 1] == b'`' {
            diagnostics.push(Diagnostic::new(
                WARNING_34,
                DiagnosticKind::Message,
                "",
                0,
                pos,
                1,
                "Don't mix quotes.",
                Vec::new(),
            ));
        }
        if b == b'`' && pos + 1 < line.len() && line[pos + 1] == b'\'' {
            diagnostics.push(Diagnostic::new(
                WARNING_34,
                DiagnosticKind::Message,
                "",
                0,
                pos,
                1,
                "Don't mix quotes.",
                Vec::new(),
            ));
        }
    }
    diagnostics
}

/// Warning 32: Use ` to begin quotation, not '.
/// Warning 33: Use ' to end quotation, not `.
/// Warning 34: Don't mix quotes.
/// Warning 38: Quote style (punctuation in/out of quotes)
fn warning_32_33_34_38_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_math = initial_math;
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }
        match token.kind {
            TokenKind::Punctuation(b'\'') => {
                // W32: ' before alpha, preceded by space/punct → using ' as open quote
                let prev_ok = i > 0
                    && matches!(
                        tokens[i - 1].kind,
                        TokenKind::Space | TokenKind::Punctuation(_)
                    );
                let next_is_alpha = tokens.get(i + 1).is_some_and(|t| {
                    if let TokenKind::Text(tx) = &t.kind {
                        !tx.is_empty() && tx[0].is_ascii_alphabetic()
                    } else {
                        false
                    }
                });
                let next_pair_is_alpha = matches!(
                    tokens.get(i + 1).map(|token| &token.kind),
                    Some(TokenKind::Punctuation(b'\''))
                ) && tokens.get(i + 2).is_some_and(|t| {
                    if let TokenKind::Text(tx) = &t.kind {
                        !tx.is_empty() && tx[0].is_ascii_alphabetic()
                    } else {
                        false
                    }
                });
                let previous_is_same = matches!(
                    tokens.get(i.wrapping_sub(1)).map(|token| &token.kind),
                    Some(TokenKind::Punctuation(b'\''))
                );
                if prev_ok
                    && (next_is_alpha || next_pair_is_alpha)
                    && !previous_is_same
                    && config.warning_32_enabled
                {
                    diagnostics.push(Diagnostic::new(
                        WARNING_32,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        if next_pair_is_alpha { 2 } else { 1 },
                        "Use ` to begin quotation, not '.",
                        Vec::new(),
                    ));
                }
            }
            TokenKind::Punctuation(b'`') => {
                // W38: handled in warning_38_line_check
                // W33: ` after alpha, followed by punct → using ` as close quote
                let prev_is_alpha = i > 0 && {
                    if let TokenKind::Text(tx) = &tokens[i - 1].kind {
                        tx.last().copied().is_some_and(|c| c.is_ascii_alphabetic())
                    } else {
                        false
                    }
                };
                let next_ok = tokens.get(i + 1).map_or(true, |t| {
                    matches!(
                        t.kind,
                        TokenKind::Space | TokenKind::Punctuation(_) | TokenKind::EndGroup
                    )
                });
                let next_is_same = matches!(
                    tokens.get(i + 1).map(|token| &token.kind),
                    Some(TokenKind::Punctuation(b'`'))
                );
                let previous_is_same = matches!(
                    tokens.get(i.wrapping_sub(1)).map(|token| &token.kind),
                    Some(TokenKind::Punctuation(b'`'))
                );
                if prev_is_alpha && next_ok && !previous_is_same && config.warning_33_enabled {
                    diagnostics.push(Diagnostic::new(
                        WARNING_33,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        if next_is_same { 2 } else { 1 },
                        "Use ' to end quotation, not `.",
                        Vec::new(),
                    ));
                }
            }
            _ => {}
        }
    }
    diagnostics
}

/// Italic correction check: W4 (found in non-italic), W5 (duplicate), W6 (not found), W28 (small punct)
fn italic_correction_check(
    tokens: &[Token],
    state: &mut CheckState,
    config: &CheckerConfig,
    file: &str,
    line_no: i64,
    line: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Group italic flags: Vec of (entered_with_italic_flag) per group level
    // `true` means the group was entered with italic flag ON
    let mut group_flags: Vec<bool> = Vec::new();
    // Pending ItFlag set by ItalCmd (\textit{ etc) before the next `{`
    let mut pending_itflag: Option<bool> = None;

    for token in tokens {
        match &token.kind {
            TokenKind::BeginGroup | TokenKind::BeginOptional => {
                // Determine what to save for this group
                // Default: save whether we were in italic (ItState=1) or corrected (ItState=2)
                let group_flag = pending_itflag
                    .take()
                    .unwrap_or(state.italic_state == 1 || state.italic_state == 2);
                group_flags.push(group_flag);
            }
            TokenKind::EndGroup | TokenKind::EndOptional => {
                // W6: missing italic correction — efNoItal group + itOn + no small punct
                let saved_flag = group_flags.pop();
                if let Some(had_italic_flag) = saved_flag {
                    if !had_italic_flag && state.italic_state == 1 && config.warning_6_enabled {
                        // Small punctuation immediately before or after the group suppresses W6.
                        let before_close = token.span.start.saturating_sub(1);
                        let after_close = token.span.end;
                        if (token.span.start == 0
                            || !matches!(line.get(before_close), Some(b'.' | b',')))
                            && !matches!(line.get(after_close), Some(b'.' | b','))
                        {
                            diagnostics.push(Diagnostic::new(
                                6,
                                DiagnosticKind::Warning,
                                file,
                                line_no,
                                token.span.start,
                                1,
                                "No italic correction (`\\/') found.",
                                line.to_vec(),
                            ));
                        }
                    }
                    state.italic_state = if had_italic_flag { 1 } else { 0 };
                }
            }
            TokenKind::Command(cmd) => {
                // Check Italic commands: \it, \em, \sl etc. — switch to italic
                if config.italic_commands.iter().any(|ic| ic == cmd.as_slice()) {
                    state.italic_state = 1;
                }
                // Check NonItalic commands: \bf, \rm, \sf etc. — switch off italic
                if config
                    .non_italic_commands
                    .iter()
                    .any(|nic| nic == cmd.as_slice())
                {
                    state.italic_state = 0;
                }
                // \/ (italic correction) handling
                if cmd == br"\/" {
                    match state.italic_state {
                        0 => {
                            if config.warning_4_enabled {
                                diagnostics.push(Diagnostic::new(
                                    4,
                                    DiagnosticKind::Warning,
                                    file,
                                    line_no,
                                    token.span.start,
                                    2,
                                    "Italic correction (`\\/') found in non-italic buffer.",
                                    line.to_vec(),
                                ));
                            }
                        }
                        1 => {
                            state.italic_state = 2; // itCorrected (upstream: only for itOn case)
                            // W28: \/ before small punctuation (. or ,)
                            let mut pos = token.span.end;
                            while pos < line.len() && matches!(line[pos], b'{' | b'}') {
                                pos += 1;
                            }
                            if pos < line.len() && matches!(line[pos], b'.' | b',') {
                                if config.warning_28_enabled {
                                    diagnostics.push(Diagnostic::new(
                                        28,
                                        DiagnosticKind::Warning,
                                        file,
                                        line_no,
                                        token.span.start,
                                        2,
                                        "Don't use \\/ in front of small punctuation.",
                                        line.to_vec(),
                                    ));
                                }
                            }
                        }
                        2 => {
                            if config.warning_5_enabled {
                                diagnostics.push(Diagnostic::new(
                                    5,
                                    DiagnosticKind::Warning,
                                    file,
                                    line_no,
                                    token.span.start,
                                    2,
                                    "Italic correction (`\\/') found more than once.",
                                    line.to_vec(),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        // After processing a command token, check if next token is BeginGroup for ItalCmd
        // This is handled by the next iteration encountering BeginGroup
    }
}

/// Warning 7: Accent command needs \i or \j.
fn warning_7_check(tokens: &[Token], config: &CheckerConfig, _math_mode: bool) -> Vec<Diagnostic> {
    if !config.warning_7_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let TokenKind::Command(cmd) = &token.kind else {
            continue;
        };
        if !matches_any(cmd, &config.ij_accent) {
            continue;
        }
        // Check if accent followed by i or j (possibly through braces)
        let next_idx = {
            let mut ni = i + 1;
            // Skip BeginGroup token if present (e.g., \hat{j})
            if tokens
                .get(ni)
                .is_some_and(|t| matches!(t.kind, TokenKind::BeginGroup))
            {
                ni += 1;
            }
            ni
        };
        let Some(next) = tokens.get(next_idx) else {
            continue;
        };
        let TokenKind::Text(text) = &next.kind else {
            continue;
        };
        if text.first() != Some(&b'i') && text.first() != Some(&b'j') {
            continue;
        }
        let accent_name = std::str::from_utf8(cmd).unwrap_or("\\?");
        let needs = if text.first() == Some(&b'i') {
            "\\i"
        } else {
            "\\jmath"
        };
        let msg = format!("Accent command `{accent_name}' needs use of `{needs}'.");
        diagnostics.push(Diagnostic::new(
            WARNING_7,
            DiagnosticKind::Message,
            "",
            0,
            token.span.start,
            cmd.len(),
            msg,
            Vec::new(),
        ));
    }
    diagnostics
}

/// Warning 8: Wrong length of dash may have been used.
fn warning_8_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
) -> Vec<Diagnostic> {
    if !config.warning_8_enabled {
        return Vec::new();
    }
    let mut in_math = initial_math;
    let mut diagnostics = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        if matches!(tokens[i].kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            i += 1;
            continue;
        }
        if !matches!(tokens[i].kind, TokenKind::Punctuation(b'-')) {
            i += 1;
            continue;
        }

        let start = i;
        while i < tokens.len() && matches!(tokens[i].kind, TokenKind::Punctuation(b'-')) {
            i += 1;
        }
        let dash_count = (i - start) as i64;

        let prev_token = if start > 0 {
            Some(&tokens[start - 1])
        } else {
            None
        };
        let next_token = tokens.get(i);

        let prev_char = prev_token.and_then(|t| match &t.kind {
            TokenKind::Text(t) => t.last().copied(),
            TokenKind::Punctuation(c) => Some(*c),
            TokenKind::Space => Some(b' '),
            _ => None,
        });
        let next_char = next_token.and_then(|t| match &t.kind {
            TokenKind::Text(t) => t.first().copied(),
            TokenKind::Punctuation(c) => Some(*c),
            TokenKind::Space => Some(b' '),
            _ => None,
        });

        let is_valid = if in_math {
            dash_count <= 1
        } else {
            match (prev_char, next_char) {
                (Some(p), Some(n)) if p.is_ascii_alphabetic() && n.is_ascii_alphabetic() => {
                    config.hyph_dash.contains(&dash_count)
                }
                (Some(p), Some(n)) if p.is_ascii_digit() && n.is_ascii_digit() => {
                    config.num_dash.contains(&dash_count)
                }
                (Some(b' '), Some(b' ')) => config.word_dash.contains(&dash_count),
                _ => true,
            }
        };

        // Skip dashes inside WipeArg command arguments (these are wiped upstream)
        let inside_wiped = (0..start).any(|j| {
            if let TokenKind::Command(cmd) = &tokens[j].kind {
                config.wipe_arg.iter().any(|wa| {
                    cmd == &wa.command.as_slice() && (j + 3 < tokens.len()) // at least some tokens after the command
                })
            } else {
                false
            }
        });
        if inside_wiped {
            continue; // Skip — upstream wipes these arguments
        }

        let mut invalid_by_exception = false;
        if is_valid && !config.dash_excpt.is_empty() {
            let mut phrase = Vec::new();
            for j in (0..start).rev() {
                if let TokenKind::Text(t) = &tokens[j].kind {
                    phrase.splice(0..0, t.iter().copied());
                } else if matches!(tokens[j].kind, TokenKind::Punctuation(b'-')) {
                    phrase.splice(0..0, [b'-']);
                } else {
                    break;
                }
                if phrase.len() > 40 {
                    break;
                }
            }
            let current_dash_offset = phrase.len();
            for _ in 0..dash_count {
                phrase.push(b'-');
            }
            for token in tokens.iter().skip(i) {
                if let TokenKind::Text(t) = &token.kind {
                    phrase.extend(t);
                } else if matches!(token.kind, TokenKind::Punctuation(b'-')) {
                    phrase.push(b'-');
                } else {
                    break;
                }
                if phrase.len() > 40 {
                    break;
                }
            }

            for exc in &config.dash_excpt {
                let collapsed = collapse_dash_runs(exc);
                for match_start in phrase
                    .windows(collapsed.len())
                    .enumerate()
                    .filter_map(|(pos, w)| (w == collapsed.as_slice()).then_some(pos))
                {
                    if current_dash_offset < match_start
                        || current_dash_offset >= match_start + collapsed.len()
                    {
                        continue;
                    }
                    let rel = current_dash_offset - match_start;
                    if dash_run_len_at_collapsed(exc, rel)
                        .is_some_and(|expected| expected != dash_count as usize)
                    {
                        invalid_by_exception = true;
                        break;
                    }
                }
                if invalid_by_exception {
                    break;
                }
            }
        }

        if !is_valid || invalid_by_exception {
            // Check DashExcpt: if dash sequence matches an exception, skip warning
            // Build the phrase around the dashes
            if !invalid_by_exception && !config.dash_excpt.is_empty() {
                let mut phrase = Vec::new();
                // Collect text before dashes
                for j in (0..start).rev() {
                    if let TokenKind::Text(t) = &tokens[j].kind {
                        phrase.splice(0..0, t.iter().copied());
                    } else if matches!(tokens[j].kind, TokenKind::Punctuation(b'-')) {
                        phrase.push(b'-');
                    } else {
                        break;
                    }
                    if phrase.len() > 40 {
                        break;
                    }
                }
                // Add the dashes
                for _ in 0..dash_count {
                    phrase.push(b'-');
                }
                // Collect text after dashes
                for j in i..tokens.len() {
                    if let TokenKind::Text(t) = &tokens[j].kind {
                        phrase.extend(t);
                    } else if matches!(tokens[j].kind, TokenKind::Punctuation(b'-')) {
                        phrase.push(b'-');
                    } else {
                        break;
                    }
                    if phrase.len() > 40 {
                        break;
                    }
                }
                // Check if any exception matches
                let mut skip = false;
                for exc in &config.dash_excpt {
                    if phrase.windows(exc.len()).any(|w| w == exc.as_slice()) {
                        skip = true;
                        break;
                    }
                }
                if skip {
                    continue;
                }
            }

            diagnostics.push(Diagnostic::new(
                WARNING_8,
                DiagnosticKind::Message,
                "",
                0,
                tokens[start].span.start,
                dash_count as usize,
                "Wrong length of dash may have been used.",
                Vec::new(),
            ));
        }
    }

    diagnostics
}

/// Check for matching brackets and warn on mismatches (warning 9/10).
fn check_brackets(
    state: &mut CheckState,
    tokens: &[Token],
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
    diagnostics: &mut Vec<Diagnostic>,
    s: &LineSuppressions,
    fs: &HashSet<i64>,
) {
    let open_match = |c: u8| -> Option<u8> {
        match c {
            b'}' => Some(b'{'),
            b']' => Some(b'['),
            b')' => Some(b'('),
            _ => None,
        }
    };
    let close_match = |c: u8| -> Option<u8> {
        match c {
            b'{' => Some(b'}'),
            b'[' => Some(b']'),
            b'(' => Some(b')'),
            _ => None,
        }
    };
    let is_open = |c: u8| -> bool { matches!(c, b'{' | b'[' | b'(') };
    let is_close = |c: u8| -> bool { matches!(c, b'}' | b']' | b')') };

    for (index, token) in tokens.iter().enumerate() {
        if is_inside_verb_or_wiped_arg(tokens, index, config) {
            continue;
        }
        let ch = match token.kind {
            TokenKind::BeginGroup => b'{',
            TokenKind::EndGroup => b'}',
            TokenKind::BeginOptional => b'[',
            TokenKind::EndOptional => b']',
            TokenKind::Punctuation(b'(') => b'(',
            TokenKind::Punctuation(b')') => b')',
            _ => continue,
        };

        if is_open(ch) {
            state.bracket_stack.push((ch, token.span.start));
        } else if is_close(ch) {
            let expected = open_match(ch).unwrap();
            if let Some((top_ch, _)) = state.bracket_stack.pop() {
                if top_ch != expected {
                    // Mismatch: '}' expected, found ']'
                    if !is_suppressed(WARNING_9, s, fs) && config.warning_9_enabled {
                        let found = &[ch as char];
                        let exp = &[close_match(top_ch).unwrap_or(expected) as char];
                        let msg = format!(
                            "`{}' expected, found `{}'.",
                            exp.iter().collect::<String>(),
                            found.iter().collect::<String>()
                        );
                        diagnostics.push(Diagnostic::new(
                            WARNING_9,
                            DiagnosticKind::Message,
                            file,
                            line_no,
                            token.span.start,
                            1,
                            msg,
                            line.to_vec(),
                        ));
                    }
                }
            } else {
                // Solo close bracket
                if !is_suppressed(WARNING_10, s, fs) && config.warning_10_enabled {
                    let found = &[ch as char];
                    let msg = format!("Solo `{}' found.", found.iter().collect::<String>());
                    diagnostics.push(Diagnostic::new(
                        WARNING_10,
                        DiagnosticKind::Message,
                        file,
                        line_no,
                        token.span.start,
                        1,
                        msg,
                        line.to_vec(),
                    ));
                }
            }
        }
    }
}

/// Warning 11: Ellipsis (...) vs \dots
fn warning_11_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_11_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    // Look for three consecutive Punctuation(b'.') tokens
    for (i, token) in tokens.iter().enumerate() {
        if let TokenKind::Command(cmd) = &token.kind {
            if cmd == br"\cdots" {
                let prev_is_comma =
                    i > 0 && matches!(tokens[i - 1].kind, TokenKind::Punctuation(b','));
                let next_is_comma = tokens
                    .get(i + 1)
                    .is_some_and(|t| matches!(t.kind, TokenKind::Punctuation(b',')));
                if prev_is_comma || next_is_comma {
                    diagnostics.push(Diagnostic::new(
                        WARNING_11,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        token.span.len(),
                        "You should use \\ldots to achieve an ellipsis.",
                        Vec::new(),
                    ));
                }
            } else if cmd == br"\ldots" {
                let prev_is_center = i > 0
                    && matches!(
                        tokens[i - 1].kind,
                        TokenKind::Command(ref prev) if prev == br"\cdot"
                    );
                let next_is_center = tokens.get(i + 1).is_some_and(
                    |t| matches!(t.kind, TokenKind::Command(ref next) if next == br"\cdot"),
                );
                if prev_is_center || next_is_center {
                    diagnostics.push(Diagnostic::new(
                        WARNING_11,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        token.span.len(),
                        "You should use \\cdots to achieve an ellipsis.",
                        Vec::new(),
                    ));
                }
            }
        }

        if token.kind != TokenKind::Punctuation(b'.') {
            continue;
        }
        if tokens
            .get(i + 1)
            .map_or(true, |t| t.kind != TokenKind::Punctuation(b'.'))
        {
            continue;
        }
        if tokens
            .get(i + 2)
            .map_or(true, |t| t.kind != TokenKind::Punctuation(b'.'))
        {
            continue;
        }
        // Found three consecutive dots
        // Check character before/after the dots to choose message (matching upstream CheckDots)
        let prev_char = if i > 0 {
            match &tokens[i - 1].kind {
                TokenKind::Punctuation(c) => Some(*c),
                TokenKind::Text(t) => t.last().copied(),
                _ => None,
            }
        } else {
            None
        };
        let next_char = if i + 3 < tokens.len() {
            match &tokens[i + 3].kind {
                TokenKind::Punctuation(c) => Some(*c),
                TokenKind::Text(t) => t.first().copied(),
                _ => None,
            }
        } else {
            None
        };

        let center_dots_chars = b"=+-\\cdot\\div&\\times\\geq\\leq<>";
        let is_center = prev_char.map_or(false, |c| center_dots_chars.contains(&c))
            || next_char.map_or(false, |c| center_dots_chars.contains(&c));

        let msg = if is_math_context(tokens, i) && is_center {
            "You should use \\cdots to achieve an ellipsis."
        } else {
            "You should use \\ldots to achieve an ellipsis."
        };
        diagnostics.push(Diagnostic::new(
            WARNING_11,
            DiagnosticKind::Message,
            "",
            0,
            token.span.start,
            3,
            msg,
            Vec::new(),
        ));
    }
    diagnostics
}

fn is_math_context(tokens: &[Token], index: usize) -> bool {
    // Check if this token is inside math mode (between $...$)
    let mut math = false;
    for token in tokens.iter().take(index) {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            math = !math;
        }
    }
    math
}

/// Warning 21: This command might not be intended.
/// Fires when a command is immediately followed by \X where X is a special char.
fn warning_21_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_21_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let TokenKind::Command(cmd) = &token.kind else {
            continue;
        };
        // Command must be \ + 1 special char (not letter, not space)
        if cmd.len() != 2 {
            continue;
        }
        let ch = cmd[1];
        if ch.is_ascii_alphabetic() || ch == b' ' {
            continue;
        }
        // Must be preceded by another command (not a space)
        if i == 0 {
            continue;
        }
        if !matches!(tokens[i - 1].kind, TokenKind::Command(_)) {
            continue;
        }
        diagnostics.push(Diagnostic::new(
            WARNING_21,
            DiagnosticKind::Message,
            "",
            0,
            token.span.start,
            2,
            "This command might not be intended.",
            Vec::new(),
        ));
    }
    diagnostics
}

/// Warning 22: Comment displayed.
fn warning_22_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_22_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for token in tokens {
        if matches!(token.kind, TokenKind::Comment(_)) {
            diagnostics.push(Diagnostic::new(
                WARNING_22,
                DiagnosticKind::Message,
                "",
                0,
                token.span.start,
                1,
                "Comment displayed.",
                Vec::new(),
            ));
        }
    }
    diagnostics
}

/// Warning 24: Delete this space to maintain correct pagereferences (PostLink space).
fn warning_24_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_24_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token.kind, TokenKind::Space) {
            continue;
        }
        if let Some(next) = tokens.get(i + 1) {
            if let Some(cmd) = command_name(next) {
                if matches_any(cmd, &config.post_link) {
                    let _name = std::str::from_utf8(cmd).unwrap_or("\\?");
                    diagnostics.push(Diagnostic::new(
                        WARNING_24,
                        DiagnosticKind::Message,
                        "",
                        0,
                        if token.span.start == 0 {
                            token.span.end.saturating_sub(1)
                        } else {
                            token.span.start
                        },
                        1,
                        "Delete this space to maintain correct pagereferences.",
                        Vec::new(),
                    ));
                }
            }
        }
    }
    diagnostics
}

/// Warning 25: You might wish to put this between a pair of `{}'
fn warning_25_check(tokens: &[Token], _config: &CheckerConfig) -> Vec<Diagnostic> {
    // Check for ^ or _ followed by multiple chars without braces
    // e.g., 10^10 should be 10^{10}
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(
            token.kind,
            TokenKind::Punctuation(b'^') | TokenKind::Punctuation(b'_')
        ) {
            continue;
        }
        // If next token is not {, this is a missing brace case
        // But only if the next token is a single character
        match tokens.get(i + 1) {
            Some(Token {
                kind: TokenKind::BeginGroup,
                ..
            }) => continue,
            Some(Token {
                kind: TokenKind::Text(t),
                ..
            }) if t.len() > 1 => {
                diagnostics.push(Diagnostic::new(
                    WARNING_25,
                    DiagnosticKind::Message,
                    "",
                    0,
                    token.span.end,
                    t.len(),
                    "You might wish to put this between a pair of `{}'",
                    Vec::new(),
                ));
            }
            _ => {}
        }
    }
    diagnostics
}

/// Warning 26: You ought to remove spaces in front of punctuation.
fn warning_26_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
) -> Vec<Diagnostic> {
    if !config.warning_26_enabled {
        return Vec::new();
    }
    let mut in_math = initial_math;
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }
        if !matches!(token.kind, TokenKind::Space) {
            continue;
        }
        // Check if next token is ? or !
        let next = tokens.get(i + 1);
        if let Some(
            Token {
                kind: TokenKind::Punctuation(b'?'),
                span,
                ..
            }
            | Token {
                kind: TokenKind::Punctuation(b'!'),
                span,
                ..
            },
        ) = next
        {
            diagnostics.push(Diagnostic::new(
                WARNING_26,
                DiagnosticKind::Message,
                "",
                0,
                span.start.saturating_sub(1),
                1,
                "You ought to remove spaces in front of punctuation.",
                Vec::new(),
            ));
        }
    }
    diagnostics
}

/// Warning 27: Could not execute LaTeX command (file not found)
fn warning_27_check(tokens: &[Token], config: &CheckerConfig, _file: &str) -> Vec<Diagnostic> {
    if !config.warning_27_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let TokenKind::Command(cmd) = &token.kind else {
            continue;
        };
        if cmd != br"\input" && cmd != br"\include" {
            continue;
        }
        // Check for argument and verify file exists (skip spaces)
        let mut arg_idx = i + 1;
        if let Some(t) = tokens.get(arg_idx) {
            if matches!(&t.kind, TokenKind::Space) {
                arg_idx += 1;
            }
        }
        let arg_token = tokens.get(arg_idx);
        let arg_text = if let Some(t) = arg_token {
            match &t.kind {
                TokenKind::BeginGroup => tokens.get(arg_idx + 1).and_then(|t2| {
                    if let TokenKind::Text(name) = &t2.kind {
                        Some(name)
                    } else {
                        None
                    }
                }),
                TokenKind::Text(name) => Some(name),
                _ => None,
            }
        } else {
            None
        };
        if let Some(name) = arg_text {
            let filename = std::str::from_utf8(name).unwrap_or("");
            if !filename.is_empty() {
                let full_path = std::path::Path::new(filename);
                let mut tex_path = full_path.as_os_str().to_os_string();
                tex_path.push(".tex");
                if !full_path.is_file() && !std::path::Path::new(&tex_path).is_file() {
                    diagnostics.push(Diagnostic::new(
                        27,
                        config.warning_kind(WARNING_27),
                        "",
                        0,
                        token.span.start,
                        cmd.len(),
                        "Could not execute LaTeX command.",
                        Vec::new(),
                    ));
                }
            }
        }
    }
    diagnostics
}

/// Warning 14: Could not find argument for command.
fn warning_14_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_14_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    let input_cmds: &[&[u8]] = &[
        br"\input",
        br"\include",
        br"\includeonly",
        br"\hat",
        br"\verb",
    ];
    for (i, token) in tokens.iter().enumerate() {
        let TokenKind::Command(cmd) = &token.kind else {
            continue;
        };
        if !input_cmds.iter().any(|c| *c == cmd.as_slice()) {
            continue;
        }

        // Check for argument: {text} or \input file or \input[opt]{file}
        let has_arg = {
            let n1 = tokens.get(i + 1);
            let n2 = tokens.get(i + 2);
            // Direct argument: \input{file} or \input file or \input[
            (n1.is_some() && matches!(&n1.unwrap().kind,
                TokenKind::BeginGroup | TokenKind::Text(_) | TokenKind::BeginOptional))
            // Space-prefixed argument: \input file or \input {file}
            || (n1.is_some() && matches!(&n1.unwrap().kind, TokenKind::Space)
                && n2.is_some() && matches!(&n2.unwrap().kind,
                    TokenKind::Text(_) | TokenKind::BeginGroup | TokenKind::BeginOptional))
        };

        // Special check for \verb: if followed by delimiter but no matching closer
        if has_arg && cmd == br"\verb" && tokens.get(i + 1).is_some() {
            if let TokenKind::Punctuation(delim) = tokens[i + 1].kind {
                // Check if there's a matching closing delimiter in the tokens
                let has_close = tokens[(i + 2)..]
                    .iter()
                    .any(|t| matches!(t.kind, TokenKind::Punctuation(d) if d == delim));
                if !has_close {
                    diagnostics.push(Diagnostic::new(
                        WARNING_14,
                        DiagnosticKind::Error,
                        "",
                        0,
                        token.span.start,
                        cmd.len(),
                        "Could not find argument for command.",
                        Vec::new(),
                    ));
                }
            }
        }

        if !has_arg {
            diagnostics.push(Diagnostic::new(
                WARNING_14,
                DiagnosticKind::Error,
                "",
                0,
                token.span.start,
                cmd.len(),
                "Could not find argument for command.",
                Vec::new(),
            ));
        }
    }
    diagnostics
}

/// Warning 29: $\times$ may look prettier here.
fn warning_29_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_29_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    // Look for patterns like "640x200" in text
    for token in tokens.iter() {
        if let TokenKind::Text(text) = &token.kind {
            // Check for "digit x digit" pattern
            if let Ok(s) = std::str::from_utf8(text) {
                if s.contains('x') && s.chars().any(|c| c.is_ascii_digit()) {
                    // Find if x appears between digits
                    let bytes = s.as_bytes();
                    for (j, &b) in bytes.iter().enumerate() {
                        if b == b'x'
                            && j > 0
                            && j + 1 < bytes.len()
                            && bytes[j - 1].is_ascii_digit()
                            && bytes[j + 1].is_ascii_digit()
                        {
                            diagnostics.push(Diagnostic::new(
                                WARNING_29,
                                DiagnosticKind::Message,
                                "",
                                0,
                                token.span.start + j,
                                1,
                                "$\\times$ may look prettier here.",
                                Vec::new(),
                            ));
                        }
                    }
                }
            }
        }
    }
    diagnostics
}

/// Warning 30: Multiple spaces detected in input.
/// Only reports spaces that are NOT at line boundaries (matching upstream).
fn warning_30_check(
    tokens: &[Token],
    config: &CheckerConfig,
    line_len: usize,
    initial_math: bool,
    _skip_leading_spaces: bool,
) -> Vec<Diagnostic> {
    if !config.warning_30_enabled {
        return Vec::new();
    }
    let mut in_math = initial_math;
    let mut diagnostics = Vec::new();
    for token in tokens {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }
        if !matches!(token.kind, TokenKind::Space) || token.span.len() <= 1 {
            continue;
        }
        // Skip trailing spaces (at end of line — the normalization-appended space)
        if token.span.end >= line_len {
            continue;
        }
        // Skip leading spaces when SeenSpace=false (after comments) or when
        // there are only 2 spaces (upstream: first has PrePtr=null → skip,
        // second needs TmpPtr-BufPtr>0 requiring 3+ total)
        if token.span.start == 0 && (_skip_leading_spaces || token.span.len() < 3) {
            continue;
        }
        diagnostics.push(Diagnostic::new(
            WARNING_30,
            DiagnosticKind::Message,
            "",
            0,
            if token.span.start == 0 {
                1
            } else {
                token.span.start
            },
            if token.span.start == 0 {
                token.span.len() - 1
            } else {
                token.span.len()
            },
            "Multiple spaces detected in input.",
            Vec::new(),
        ));
    }
    diagnostics
}

/// Warning 35: You should perhaps use \sin, \cos etc.
fn warning_35_check(
    tokens: &[Token],
    config: &CheckerConfig,
    math_modes: &[bool],
) -> Vec<Diagnostic> {
    if !config.warning_35_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) || !math_modes[index] {
            continue;
        }
        if let TokenKind::Text(text) = &token.kind {
            if matches!(
                tokens.get(index + 1).map(|token| &token.kind),
                Some(TokenKind::Punctuation(b'('))
            ) {
                continue;
            }
            for roman in &config.math_roman {
                if let Ok(roman_str) = std::str::from_utf8(roman) {
                    if text == roman.as_slice() {
                        diagnostics.push(Diagnostic::new(
                            WARNING_35,
                            DiagnosticKind::Message,
                            "",
                            0,
                            token.span.start,
                            text.len(),
                            format!("You should perhaps use `\\{roman_str}' instead."),
                            Vec::new(),
                        ));
                        break;
                    }
                }
            }
        }
    }
    diagnostics
}

/// Warning 12: Interword spacing (`\ ') should perhaps be used.
/// Warning 13: Intersentence spacing (`\@') should perhaps be used.
fn warning_12_13_check(
    tokens: &[Token],
    config: &CheckerConfig,
    initial_math: bool,
    french_spacing: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_math = initial_math;
    for (i, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            in_math = !in_math;
            continue;
        }
        if in_math {
            continue;
        }
        if matches!(token.kind, TokenKind::Punctuation(b'!' | b'?'))
            && i > 0
            && matches!(tokens[i - 1].kind, TokenKind::Punctuation(b'.'))
        {
            continue;
        }

        // Check for end-of-sentence punctuation (. ! ?)
        let is_eos = matches!(
            token.kind,
            TokenKind::Punctuation(b'.')
                | TokenKind::Punctuation(b'!')
                | TokenKind::Punctuation(b'?')
        );
        if !is_eos {
            continue;
        }

        // Check for ... → covered by warning 11, skip
        if i + 2 < tokens.len()
            && matches!(tokens[i + 1].kind, TokenKind::Punctuation(b'.'))
            && matches!(tokens[i + 2].kind, TokenKind::Punctuation(b'.'))
        {
            continue;
        }

        // Skip past end-of-sentence punctuation (! ? : ;) after the dot
        let mut scan = i + 1;
        while let Some(t) = tokens.get(scan) {
            if matches!(t.kind, TokenKind::Punctuation(b'!' | b'?' | b':')) {
                scan += 1;
            } else {
                break;
            }
        }

        let Some(next) = tokens.get(scan) else {
            continue;
        };
        if !matches!(next.kind, TokenKind::Space) {
            continue;
        }

        let Some(after_space) = tokens.get(scan + 1) else {
            continue;
        };

        // Build the word before the dot by scanning backwards
        let prev_word = if i > 0 {
            let mut word = Vec::new();
            for j in (0..i).rev() {
                match &tokens[j].kind {
                    TokenKind::Text(t) => {
                        word.extend(t.iter().rev());
                        if word.len() > 20 {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            word.reverse();
            if word.is_empty() { None } else { Some(word) }
        } else {
            None
        };

        // Check if this is an abbreviation
        let is_abbrev = prev_word.as_ref().map_or(false, |w| {
            let mut punctuated_word = w.clone();
            punctuated_word.push(b'.');
            let w_len = punctuated_word.len();
            config.abbrev.iter().any(|a| {
                let a_suffix = if a.len() > w_len {
                    &a[a.len() - w_len..]
                } else {
                    a.as_slice()
                };
                a_suffix == punctuated_word.as_slice()
            }) || config.abbrev_case.iter().any(|a| {
                let a_lower: Vec<u8> = a.iter().map(|b| b.to_ascii_lowercase()).collect();
                let w_lower: Vec<u8> = punctuated_word
                    .iter()
                    .map(|b| b.to_ascii_lowercase())
                    .collect();
                let a_suffix = if a_lower.len() > w_lower.len() {
                    &a_lower[a_lower.len() - w_lower.len()..]
                } else {
                    &a_lower
                };
                a_suffix == w_lower.as_slice()
            })
        });

        // Character before the dot
        let prev_char = if i > 0 {
            match &tokens[i - 1].kind {
                TokenKind::Text(t) => t.last().copied(),
                TokenKind::Command(cmd) => command_word(cmd).last().copied(),
                _ => None,
            }
        } else {
            None
        };
        let prev_command = if i > 0 {
            match &tokens[i - 1].kind {
                TokenKind::Command(cmd) => Some(command_word(cmd)),
                _ => None,
            }
        } else {
            None
        };

        match &after_space.kind {
            TokenKind::Text(t) if !t.is_empty() => {
                let first = t[0];
                if first == b'`' && is_abbrev {
                    if config.warning_12_enabled {
                        diagnostics.push(Diagnostic::new(
                            WARNING_12,
                            DiagnosticKind::Message,
                            "",
                            0,
                            token.span.start,
                            1,
                            "Interword spacing (`\\ ') should perhaps be used.",
                            Vec::new(),
                        ));
                    }
                } else if first.is_ascii_lowercase() {
                    // lowercase after space after dot → interword spacing
                    // Skip if preceded by uppercase (e.g., "Dr. foo") or known abbreviation
                    if !is_abbrev
                        && prev_char.map_or(true, |c| !c.is_ascii_uppercase())
                        && (prev_command.is_none() || config.cmd_space_style.allows_interword())
                    {
                        if config.warning_12_enabled {
                            let column = if i > 0
                                && matches!(tokens[i - 1].kind, TokenKind::Punctuation(b'.'))
                            {
                                token.span.start
                            } else {
                                token.span.start + 1
                            };
                            diagnostics.push(Diagnostic::new(
                                WARNING_12,
                                DiagnosticKind::Message,
                                "",
                                0,
                                column,
                                1,
                                "Interword spacing (`\\ ') should perhaps be used.",
                                Vec::new(),
                            ));
                        }
                    }
                } else if first.is_ascii_uppercase() {
                    if prev_word
                        .as_ref()
                        .is_some_and(|w| w.iter().any(u8::is_ascii_digit))
                    {
                        if config.warning_12_enabled {
                            diagnostics.push(Diagnostic::new(
                                WARNING_12,
                                DiagnosticKind::Message,
                                "",
                                0,
                                token.span.start + 1,
                                1,
                                "Interword spacing (`\\ ') should perhaps be used.",
                                Vec::new(),
                            ));
                        }
                        continue;
                    }

                    // Intersentence: need TWO uppercase before punctuation (e.g., "FOO. Bar")
                    // Check token before the dot — must be Text ending with two uppercase
                    if !french_spacing {
                        if let Some(prev_token) = tokens.get(i.saturating_sub(1)) {
                            if let TokenKind::Text(t) = &prev_token.kind {
                                let t_len = t.len();
                                if t_len >= 2
                                    && t[t_len - 1].is_ascii_uppercase()
                                    && t[t_len - 2].is_ascii_uppercase()
                                    && !t.ends_with(b"\\@")
                                // \@ suppresses spacing
                                {
                                    if config.warning_13_enabled {
                                        diagnostics.push(Diagnostic::new(
                                            WARNING_13,
                                            DiagnosticKind::Message,
                                            "",
                                            0,
                                            token.span.start,
                                            1,
                                            "Intersentence spacing (`\\@') should perhaps be used.",
                                            Vec::new(),
                                        ));
                                    }
                                }
                            } else if let TokenKind::Command(cmd) = &prev_token.kind {
                                let word = command_word(cmd);
                                if config.cmd_space_style.allows_intersentence()
                                    && command_before_intersentence_punct(word, token)
                                    && config.warning_13_enabled
                                {
                                    diagnostics.push(Diagnostic::new(
                                        WARNING_13,
                                        DiagnosticKind::Message,
                                        "",
                                        0,
                                        token.span.start,
                                        1,
                                        "Intersentence spacing (`\\@') should perhaps be used.",
                                        Vec::new(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            TokenKind::Punctuation(b'`') | TokenKind::MathShift { .. } if is_abbrev => {
                if config.warning_12_enabled {
                    diagnostics.push(Diagnostic::new(
                        WARNING_12,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start + 1,
                        1,
                        "Interword spacing (`\\ ') should perhaps be used.",
                        Vec::new(),
                    ));
                }
            }
            TokenKind::Command(_) if is_abbrev => {
                if config.warning_12_enabled {
                    diagnostics.push(Diagnostic::new(
                        WARNING_12,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start + 1,
                        1,
                        "Interword spacing (`\\ ') should perhaps be used.",
                        Vec::new(),
                    ));
                }
            }
            _ => {}
        }
    }
    diagnostics
}

fn command_word(command: &[u8]) -> &[u8] {
    command.strip_prefix(b"\\").unwrap_or(command)
}

fn command_before_intersentence_punct(command: &[u8], punctuation: &Token) -> bool {
    let Some(&last) = command.last() else {
        return false;
    };
    if !last.is_ascii_uppercase() {
        return false;
    }
    let has_required_prefix = command
        .get(command.len().saturating_sub(2))
        .is_some_and(u8::is_ascii_uppercase)
        || !matches!(punctuation.kind, TokenKind::Punctuation(b'.'));
    has_required_prefix && !command.ends_with(b"\\@")
}

/// Warning 36: You should put a space (after|in front of) parenthesis.
/// Warning 37: You should avoid spaces (after|in front of) parenthesis.
fn warning_36_37_check(
    tokens: &[Token],
    config: &CheckerConfig,
    math_modes: &[bool],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if math_modes[i] {
            continue;
        }
        if is_inside_verb_or_wiped_arg(tokens, i, config) {
            continue;
        }
        match token.kind {
            TokenKind::Punctuation(b'(') => {
                // W36: no space before ( — "put a space in front of"
                if i > 0 {
                    if let Some(prev) = tokens.get(i - 1) {
                        let needs_space = match &prev.kind {
                            TokenKind::Space => false,
                            TokenKind::Punctuation(c) => {
                                !matches!(c, b'(' | b'[' | b'{' | b'`' | b'~')
                            }
                            TokenKind::Command(c) if c == br"\left" || c == br"\right" => false,
                            TokenKind::Command(c) if c.len() > 2 => true, // long cmd
                            TokenKind::Text(t) if !t.is_empty() && !t[0].is_ascii_digit() => true,
                            _ => false,
                        };
                        if needs_space && config.warning_36_enabled {
                            diagnostics.push(Diagnostic::new(
                                WARNING_36,
                                DiagnosticKind::Message,
                                "",
                                0,
                                token.span.start,
                                1,
                                "You should put a space in front of parenthesis.",
                                Vec::new(),
                            ));
                        }
                    }
                }
                // W37: space after ( — "avoid spaces after"
                if let Some(next) = tokens.get(i + 1) {
                    if matches!(next.kind, TokenKind::Space) && config.warning_37_enabled {
                        diagnostics.push(Diagnostic::new(
                            WARNING_37,
                            DiagnosticKind::Message,
                            "",
                            0,
                            next.span.start,
                            1,
                            "You should avoid spaces after parenthesis.",
                            Vec::new(),
                        ));
                    }
                }
            }
            TokenKind::Punctuation(b')') => {
                if i == 0 && config.warning_37_enabled {
                    diagnostics.push(Diagnostic::new(
                        WARNING_37,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        1,
                        "You should avoid spaces in front of parenthesis.",
                        Vec::new(),
                    ));
                }
                // W37: space before ) — "avoid spaces in front of"
                if i > 0 {
                    if let Some(prev) = tokens.get(i - 1) {
                        if matches!(prev.kind, TokenKind::Space) && config.warning_37_enabled {
                            diagnostics.push(Diagnostic::new(
                                WARNING_37,
                                DiagnosticKind::Message,
                                "",
                                0,
                                token.span.start,
                                1,
                                "You should avoid spaces in front of parenthesis.",
                                Vec::new(),
                            ));
                        }
                    }
                }
                // W36: alpha after ) — "put a space after"
                if let Some(next) = tokens.get(i + 1) {
                    if let TokenKind::Text(t) = &next.kind {
                        if !t.is_empty() && t[0].is_ascii_alphabetic() && config.warning_36_enabled
                        {
                            diagnostics.push(Diagnostic::new(
                                WARNING_36,
                                DiagnosticKind::Message,
                                "",
                                0,
                                token.span.start + 1,
                                1,
                                "You should put a space after parenthesis.",
                                Vec::new(),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    diagnostics
}

fn is_inside_verb_or_wiped_arg(tokens: &[Token], index: usize, config: &CheckerConfig) -> bool {
    for j in (0..index).rev() {
        let TokenKind::Command(cmd) = &tokens[j].kind else {
            continue;
        };
        if config.wipe_verb && cmd.starts_with(br"\verb") {
            return true;
        }
        let wipes_angle_arg = cmd.starts_with(br"\visible")
            || cmd.starts_with(br"\invisible")
            || cmd.starts_with(br"\alt")
            || cmd.starts_with(br"\temporal")
            || cmd.starts_with(br"\label")
            || config
                .wipe_arg
                .iter()
                .any(|entry| entry.command.as_slice() == cmd.as_slice() && entry.spec == b"<>");
        if !wipes_angle_arg {
            continue;
        }
        let mut saw_open = false;
        let mut closed = false;
        for token in tokens.iter().take(index).skip(j + 1) {
            match token.kind {
                TokenKind::Punctuation(b'<') => saw_open = true,
                TokenKind::Punctuation(b'>') if saw_open => {
                    closed = true;
                    break;
                }
                _ => {}
            }
        }
        if saw_open && !closed {
            return true;
        }
    }
    false
}

fn is_in_beamer_overlay(line: &[u8], column: usize) -> bool {
    for command in [
        br"\visible<".as_slice(),
        br"\invisible<".as_slice(),
        br"\alt<".as_slice(),
        br"\temporal<".as_slice(),
    ] {
        let mut start = 0usize;
        while let Some(pos) = line[start..]
            .windows(command.len())
            .position(|w| w == command)
        {
            let open = start + pos + command.len() - 1;
            let close = line[open..]
                .iter()
                .position(|&b| b == b'>')
                .map(|offset| open + offset);
            if close.is_some_and(|close| column > open && column < close) {
                return true;
            }
            start = open + 1;
        }
    }
    false
}

/// Warning 41: You ought to not use primitive TeX in LaTeX code.
fn warning_41_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_41_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for token in tokens {
        if let TokenKind::Command(cmd) = &token.kind {
            if matches_any(cmd, &config.primitives) {
                diagnostics.push(Diagnostic::new(
                    WARNING_41,
                    DiagnosticKind::Message,
                    "",
                    0,
                    token.span.start,
                    cmd.len(),
                    "You ought to not use primitive TeX in LaTeX code.",
                    Vec::new(),
                ));
            }
        }
    }
    diagnostics
}

/// Warning 42: You should remove spaces in front of \/ or \footnote
fn warning_42_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_42_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token.kind, TokenKind::Space) {
            continue;
        }
        if let Some(next) = tokens.get(i + 1) {
            if let Some(cmd) = command_name(next) {
                if matches_any(cmd, &config.not_pre_spaced) {
                    let cmd_name = std::str::from_utf8(cmd).unwrap_or("\\?");
                    diagnostics.push(Diagnostic::new(
                        WARNING_42,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        1,
                        format!("You should remove spaces in front of `{cmd_name}'"),
                        Vec::new(),
                    ));
                }
            }
        }
    }
    diagnostics
}

/// Warning 43: \left is normally not followed by {.
fn warning_43_check(tokens: &[Token], config: &CheckerConfig) -> Vec<Diagnostic> {
    if !config.warning_43_enabled {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let TokenKind::Command(cmd) = &token.kind else {
            continue;
        };
        // Check if this is \left or \right
        let is_left = cmd == br"\left";
        let is_right = cmd == br"\right";
        if !is_left && !is_right {
            continue;
        }

        // Find the NoCharNext entry for this command
        let for_cmd: &[u8] = if is_left { b"\\left" } else { b"\\right" };
        let bad_chars = config.no_char_next.iter().find(|(c, _)| c == for_cmd);

        // Check the next token
        if let Some(next) = tokens.get(i + 1) {
            let char_after = match &next.kind {
                TokenKind::BeginGroup => Some(b'{'),
                TokenKind::EndGroup => Some(b'}'),
                TokenKind::MathShift { display: false } => Some(b'$'),
                TokenKind::MathShift { display: true } => Some(b'$'),
                TokenKind::Punctuation(p) => Some(*p),
                // Skip escaped braces (e.g., \left\{ is correct LaTeX)
                TokenKind::Command(c) if c.len() == 2 && (c[1] == b'{' || c[1] == b'}') => None,
                TokenKind::Command(c) => c.last().copied(),
                _ => None,
            };

            if let Some(ch) = char_after {
                if let Some((_, bad)) = bad_chars {
                    if bad.contains(&ch) {
                        let name = if is_left { "\\left" } else { "\\right" };
                        diagnostics.push(Diagnostic::new(
                            WARNING_43,
                            DiagnosticKind::Message,
                            "",
                            0,
                            token.span.start,
                            cmd.len(),
                            format!("`{name}' is normally not followed by `{}'.", ch as char),
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }
    diagnostics
}

/// Warning 45: Use \[ ... \] instead of $$ ... $$.
/// Warning 46: Use \( ... \) instead of $ ... $.
/// Only fires when entering math mode (matching upstream ctOutMath context).
fn warning_45_46_check(
    tokens: &[Token],
    config: &CheckerConfig,
    math_modes: &[bool],
    line_len: usize,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        match &token.kind {
            TokenKind::MathShift { display: true } => {
                if !math_modes[i] && config.warning_45_enabled {
                    // Find closing $$ to compute full span (upstream: TmpPtr-BufPtr+4)
                    let end = tokens[i + 1..]
                        .iter()
                        .position(|t| matches!(t.kind, TokenKind::MathShift { display: true }))
                        .map(|p| tokens[i + p + 1].span.end)
                        .unwrap_or(line_len + 2);
                    diagnostics.push(Diagnostic::new(
                        WARNING_45,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        end - token.span.start,
                        "Use \\[ ... \\] instead of $$ ... $$.",
                        Vec::new(),
                    ));
                }
                i += 1;
                continue;
            }
            TokenKind::MathShift { display: false } => {
                if !math_modes[i] && config.warning_46_enabled {
                    // Find closing $ to compute full span (upstream: TmpPtr-BufPtr+2)
                    let end = tokens[i + 1..]
                        .iter()
                        .position(|t| matches!(t.kind, TokenKind::MathShift { display: false }))
                        .map(|p| tokens[i + p + 1].span.end)
                        .unwrap_or(token.span.end);
                    diagnostics.push(Diagnostic::new(
                        WARNING_46,
                        DiagnosticKind::Message,
                        "",
                        0,
                        token.span.start,
                        end - token.span.start,
                        "Use \\( ... \\) instead of $ ... $.",
                        Vec::new(),
                    ));
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    diagnostics
}

// Warnings that need more infrastructure:
// 3, 4-6, 8, 9-10, 12-13, 14, 15-16, 17, 23, 28, 31, 32-34, 36-37, 38, 40, 47, 48, 49
//
// These will be implemented in subsequent phases.

// ====== Environment tracking ======

fn update_environments(
    state: &mut CheckState,
    tokens: &[Token],
    file: &str,
    line_no: i64,
    line: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
    config: &CheckerConfig,
) {
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let TokenKind::Command(cmd) = &token.kind else {
            i += 1;
            continue;
        };
        let cmd_str = match std::str::from_utf8(cmd) {
            Ok(s) => s,
            _ => {
                i += 1;
                continue;
            }
        };

        // Detect \begin{env} pattern: \begin + { + text(env) + }
        if cmd_str == r"\begin" && i + 3 < tokens.len() {
            if matches!(tokens[i + 1].kind, TokenKind::BeginGroup) {
                if let TokenKind::Text(env_name) = &tokens[i + 2].kind {
                    if let Ok(env) = std::str::from_utf8(env_name) {
                        if matches!(tokens[i + 3].kind, TokenKind::EndGroup) {
                            state.environment_stack.push(EnvFrame {
                                name: env.to_string(),
                                line: line_no,
                                column: token.span.start,
                                len: cmd.len(),
                                source: line.to_vec(),
                            });
                            if env == "document" {
                                state.in_header = false;
                            }
                            if env_matches(env, &config.math_envirs) {
                                state.math_mode = true;
                            }
                            if env_matches(env, &config.text_envirs) {
                                state.math_mode = false;
                            }
                            if env_matches(env, &config.verb_envirs) {
                                state.in_verbatim = true;
                            }
                        }
                    }
                }
            }
        }

        // Detect \end{env} pattern: \end + { + text(env) + }
        if cmd_str == r"\end" && i + 3 < tokens.len() {
            if matches!(tokens[i + 1].kind, TokenKind::BeginGroup) {
                if let TokenKind::Text(env_name) = &tokens[i + 2].kind {
                    if let Ok(env) = std::str::from_utf8(env_name) {
                        if matches!(tokens[i + 3].kind, TokenKind::EndGroup) {
                            // Check for environment mismatch (emExpectC/emSoloC)
                            if let Some(expected) = state.environment_stack.last() {
                                if expected.name != env {
                                    diagnostics.push(Diagnostic::new(
                                        9,
                                        DiagnosticKind::Message,
                                        file,
                                        line_no,
                                        token.span.start,
                                        cmd.len(),
                                        format!("`{}' expected, found `{}'.", expected.name, env),
                                        line.to_vec(),
                                    ));
                                }
                            } else if env != "document" {
                                diagnostics.push(Diagnostic::new(
                                    10,
                                    DiagnosticKind::Message,
                                    file,
                                    line_no,
                                    token.span.start,
                                    cmd.len(),
                                    format!("Solo `{}' found.", env),
                                    line.to_vec(),
                                ));
                            }
                            state.environment_stack.pop();
                            if env_matches(env, &config.verb_envirs) {
                                state.in_verbatim = false;
                            }
                            if env_matches(env, &config.math_envirs) {
                                // Only turn off math if not inside another math environment
                                let still_in_math = state
                                    .environment_stack
                                    .iter()
                                    .any(|frame| env_matches(&frame.name, &config.math_envirs));
                                let still_in_text = state
                                    .environment_stack
                                    .iter()
                                    .any(|frame| env_matches(&frame.name, &config.text_envirs));
                                state.math_mode = still_in_math && !still_in_text;
                            }
                            if env_matches(env, &config.text_envirs) {
                                let still_in_math = state
                                    .environment_stack
                                    .iter()
                                    .any(|frame| env_matches(&frame.name, &config.math_envirs));
                                let still_in_text = state
                                    .environment_stack
                                    .iter()
                                    .any(|frame| env_matches(&frame.name, &config.text_envirs));
                                state.math_mode = still_in_math && !still_in_text;
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

fn update_math_state(state: &mut CheckState, tokens: &[Token]) {
    for token in tokens {
        match &token.kind {
            TokenKind::MathShift { display: true } => {
                state.display_math = !state.display_math;
            }
            TokenKind::MathShift { display: false } => {
                state.math_mode = !state.math_mode;
                state.command_math_mode = false;
            }
            TokenKind::Command(cmd) if cmd == br"\[" => {
                state.display_math = true;
                state.command_math_mode = true;
            }
            TokenKind::Command(cmd) if cmd == br"\]" => {
                state.display_math = false;
                state.command_math_mode = false;
            }
            TokenKind::Command(cmd) if cmd == br"\(" => {
                state.math_mode = true;
                state.command_math_mode = true;
            }
            TokenKind::Command(cmd) if cmd == br"\)" => {
                state.math_mode = false;
                state.command_math_mode = false;
            }
            _ => {}
        }
    }
}

fn suppress_header_diagnostics(
    config: &CheckerConfig,
    line_started_in_header: bool,
    line_diagnostics_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if config.header_errors
        || !line_started_in_header
        || line_diagnostics_start >= diagnostics.len()
    {
        return;
    }
    diagnostics.truncate(line_diagnostics_start);
}

fn token_math_modes(tokens: &[Token], initial_math: bool, config: &CheckerConfig) -> Vec<bool> {
    let mut modes = Vec::with_capacity(tokens.len());
    let mut in_math = initial_math;
    let mut group_depth = 0usize;
    let mut pending_arg_mode = None;
    let mut mode_stack: Vec<(usize, bool)> = Vec::new();

    for token in tokens {
        match &token.kind {
            TokenKind::Command(cmd) => {
                modes.push(in_math);
                if config.math_commands.iter().any(|entry| entry == cmd) {
                    pending_arg_mode = Some(true);
                } else if config.text_commands.iter().any(|entry| entry == cmd) {
                    pending_arg_mode = Some(false);
                }
            }
            TokenKind::BeginGroup => {
                if let Some(arg_mode) = pending_arg_mode.take() {
                    mode_stack.push((group_depth, in_math));
                    in_math = arg_mode;
                }
                modes.push(in_math);
                group_depth += 1;
            }
            TokenKind::EndGroup => {
                modes.push(in_math);
                group_depth = group_depth.saturating_sub(1);
                if let Some(&(depth, previous_mode)) = mode_stack.last() {
                    if depth == group_depth {
                        in_math = previous_mode;
                        mode_stack.pop();
                    }
                }
            }
            TokenKind::MathShift { .. } => {
                modes.push(in_math);
                in_math = !in_math;
                pending_arg_mode = None;
            }
            TokenKind::Space => {
                modes.push(in_math);
            }
            _ => {
                modes.push(in_math);
                pending_arg_mode = None;
            }
        }
    }

    modes
}

// ====== Document checker ======

pub fn check_document(file: &str, input: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    let mut diagnostics = check_document_old(file, input, config);
    for diagnostic in &mut diagnostics {
        diagnostic.kind = config.warning_kind(diagnostic.number);
    }
    diagnostics.sort_by_key(|diag| {
        let sort_line = diag.sort_line.unwrap_or(diag.line);
        (
            sort_line == 0,
            sort_line,
            diagnostic_primary_order(diag),
            diag.column,
            diagnostic_secondary_order(diag.number),
            eof_diagnostic_order(diag),
        )
    });
    diagnostics
}

fn diagnostic_primary_order(diag: &Diagnostic) -> i32 {
    if diag.sort_line == Some(0) || diag.line == 0 {
        eof_diagnostic_order(diag)
    } else if diag.number == WARNING_1 || diag.number == WARNING_22 {
        0
    } else if diag.number == WARNING_44 {
        2
    } else {
        1
    }
}

fn diagnostic_secondary_order(number: i32) -> i32 {
    match number {
        WARNING_37 => 0,
        _ => 1,
    }
}

fn eof_diagnostic_order(diag: &Diagnostic) -> i32 {
    if diag.sort_line != Some(0) && diag.line != 0 {
        return 0;
    }
    match diag.number {
        WARNING_48 => 0,
        16 => 1,
        15 => 2,
        17 if diag.message.contains("`('") => 3,
        17 if diag.message.contains("`['") => 4,
        17 if diag.message.contains("`{'") => 5,
        _ => 6,
    }
}

/// Old token-based document checker (kept for backward compat).
pub fn check_document_old(file: &str, input: &[u8], config: &CheckerConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut state = CheckState::default();
    let eof_line = last_input_line(input);

    for (line_index, raw_line) in split_lines_preserve(input).enumerate() {
        let line_no = (line_index + 1) as i64;
        let continued_from_inline_comment = state.last_was_inline_comment;
        let line_started_in_header = state.in_header;
        let line_diagnostics_start = diagnostics.len();

        // Upstream: if (!LastWasComment) SeenSpace = TRUE
        if !state.last_was_comment {
            state.seen_space = true;
        }
        state.last_was_comment = false;
        state.last_was_inline_comment = false;

        let suppressions = if config.no_line_suppression {
            LineSuppressions::default()
        } else {
            parse_line_suppressions(raw_line)
        };
        let pending_file_suppressions = if config.no_line_suppression {
            None
        } else {
            parse_file_suppressions(raw_line)
        };

        let normalized = normalize_input_line(raw_line);
        let tokens = lex_line(&normalized);

        // Track if this line has a comment (upstream: LastWasComment)
        // Also reset SeenSpace after comment lines
        for token in &tokens {
            if let TokenKind::Command(cmd) = &token.kind {
                match cmd.as_slice() {
                    br"\frenchspacing" => state.french_spacing = true,
                    br"\nonfrenchspacing" => state.french_spacing = false,
                    _ => {}
                }
            }
            if matches!(token.kind, TokenKind::Comment(_)) {
                state.last_was_comment = true;
                state.last_was_inline_comment = token.span.start > 0;
                state.seen_space = false;
                break;
            }
        }

        // Check for per-file suppressions
        let _was_in_verbatim = state.in_verbatim;
        update_environments(
            &mut state,
            &tokens,
            file,
            line_no,
            &normalized,
            &mut diagnostics,
            config,
        );
        italic_correction_check(
            &tokens,
            &mut state,
            config,
            file,
            line_no,
            &normalized,
            &mut diagnostics,
        );
        let initial_math = state.math_mode || state.display_math;
        let math_modes = token_math_modes(&tokens, initial_math, config);
        update_math_state(&mut state, &tokens);

        // Bracket matching (warning 9/10) — run before verbatim skip
        let file_supps = state.file_suppressions.clone();
        check_brackets(
            &mut state,
            &tokens,
            file,
            line_no,
            &normalized,
            config,
            &mut diagnostics,
            &suppressions,
            &file_supps,
        );

        if state.in_verbatim {
            suppress_header_diagnostics(
                config,
                line_started_in_header,
                line_diagnostics_start,
                &mut diagnostics,
            );
            if let Some(file_supps) = pending_file_suppressions {
                state.file_suppressions = file_supps;
            }
            continue;
        }

        let s = &suppressions;
        let fs = &state.file_suppressions;

        // User warnings — skip on comment-only lines (content in % is not checked)
        let has_real_content = tokens
            .iter()
            .any(|t| !matches!(t.kind, TokenKind::Comment(_) | TokenKind::Space));
        if !is_suppressed(WARNING_20, s, fs) && config.user_warn_enabled && has_real_content {
            diagnostics.extend(check_user_warn_line(file, line_no, &normalized, config));
        }
        #[cfg(feature = "regex-bytes")]
        if !is_suppressed(WARNING_44, s, fs) && config.user_warn_regex_enabled && has_real_content {
            diagnostics.extend(check_user_warn_regex_line(
                file,
                line_no,
                &normalized,
                config,
            ));
        }

        // Run individual warnings
        if !is_suppressed(WARNING_1, s, fs) {
            for mut diag in warning_1_check(&tokens, config, initial_math) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_2, s, fs) {
            for mut diag in warning_2_check(&tokens, config, initial_math) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_3, s, fs) {
            for mut diag in warning_3_check(&normalized, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        // Warning 31: text after \end{verbatim} on same line
        if !is_suppressed(WARNING_31, s, fs) && config.warning_31_enabled {
            for (j, token) in tokens.iter().enumerate() {
                let TokenKind::Command(cmd) = &token.kind else {
                    continue;
                };
                if cmd != br"\end" {
                    continue;
                }
                // Check for \end + { + env + } pattern
                if j + 3 < tokens.len()
                    && matches!(tokens[j + 1].kind, TokenKind::BeginGroup)
                    && matches!(tokens[j + 2].kind, TokenKind::Text(_))
                    && matches!(tokens[j + 3].kind, TokenKind::EndGroup)
                {
                    if let TokenKind::Text(env_name) = &tokens[j + 2].kind {
                        if let Ok(env) = std::str::from_utf8(env_name) {
                            if env_matches(env, &config.verb_envirs) {
                                // Text after \end{env} → warning 31
                                for k in (j + 4)..tokens.len() {
                                    if !matches!(tokens[k].kind, TokenKind::Space) {
                                        diagnostics.push(Diagnostic::new(
                                            WARNING_31,
                                            DiagnosticKind::Message,
                                            file,
                                            line_no,
                                            tokens[k].span.start,
                                            normalized
                                                .len()
                                                .saturating_sub(2)
                                                .saturating_sub(tokens[k].span.start),
                                            "This text may be ignored.",
                                            normalized.clone(),
                                        ));
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !is_suppressed(WARNING_7, s, fs) {
            for mut diag in warning_7_check(&tokens, config, initial_math) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_8, s, fs) {
            for mut diag in warning_8_check(&tokens, config, initial_math) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_11, s, fs) {
            for mut diag in warning_11_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        // W40: punctuation in math mode (emPunctMath)
        if !is_suppressed(WARNING_40, s, fs) && config.warning_40_enabled {
            for (pos, &b) in normalized.iter().enumerate() {
                if pos == 0 {
                    continue;
                }
                if matches!(b, b',' | b'.' | b';' | b':') && normalized[pos - 1] != b'\\' {
                    // Check if in math mode: look for $ before the punct
                    let mut in_math = initial_math;
                    for j in 0..pos {
                        if normalized[j] == b'$' {
                            in_math = !in_math;
                        }
                    }
                    if in_math {
                        // Check if $ follows the punct (closing inline math)
                        let rest = &normalized[pos + 1..];
                        if rest.starts_with(b"$") && !rest.starts_with(b"$$") {
                            diagnostics.push(Diagnostic::new(
                                WARNING_40,
                                DiagnosticKind::Message,
                                file,
                                line_no,
                                pos,
                                1,
                                "You should put punctuation outside inner math mode.",
                                normalized.clone(),
                            ));
                        }
                    } else {
                        // Check if $$ precedes the punct (closing display math)
                        if pos >= 2 && normalized[pos - 1] == b'$' && normalized[pos - 2] == b'$' {
                            diagnostics.push(Diagnostic::new(
                                WARNING_40,
                                DiagnosticKind::Message,
                                file,
                                line_no,
                                pos,
                                1,
                                "You should put punctuation inside display math mode.",
                                normalized.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // W47/W48: ConTeXt environment matching
        // Track \start... and \stop... commands
        if !is_suppressed(47, s, fs) || !is_suppressed(48, s, fs) {
            for token in &tokens {
                if let TokenKind::Command(cmd) = &token.kind {
                    if let Ok(cmd_str) = std::str::from_utf8(cmd) {
                        if let Some(name) = cmd_str.strip_prefix("\\start") {
                            state.context_stack.push(ContextFrame {
                                name: name.to_string(),
                                line: line_no,
                                column: token.span.start + "\\start".len(),
                                source: normalized.clone(),
                            });
                            continue;
                        }
                        if let Some(name) = cmd_str.strip_prefix("\\stop") {
                            // Check ConTeXt stack
                            if let Some(frame) = state.context_stack.last() {
                                if frame.name != name {
                                    if !is_suppressed(47, s, fs) && config.warning_47_enabled {
                                        diagnostics.push(Diagnostic::new(
                                            47,
                                            DiagnosticKind::Message,
                                            file,
                                            line_no,
                                            token.span.start + "\\stop".len(),
                                            name.len(),
                                            format!(
                                                "`{}' expected, found `{}' (ConTeXt).",
                                                frame.name, name
                                            ),
                                            normalized.clone(),
                                        ));
                                    }
                                }
                                state.context_stack.pop();
                            } else {
                                if !is_suppressed(48, s, fs) && config.warning_48_enabled {
                                    diagnostics.push(Diagnostic::new(
                                        48,
                                        DiagnosticKind::Message,
                                        file,
                                        line_no,
                                        token.span.start,
                                        cmd.len(),
                                        format!("No match found for `{}' (ConTeXt).", name),
                                        normalized.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // W49: \] outside math mode
        if !is_suppressed(WARNING_49, s, fs) && config.warning_49_enabled && !initial_math {
            for token in &tokens {
                if let TokenKind::Command(cmd) = &token.kind {
                    if cmd == br"\]" {
                        diagnostics.push(Diagnostic::new(
                            49,
                            DiagnosticKind::Message,
                            file,
                            line_no,
                            token.span.start,
                            1,
                            "Expected math mode to be on here.",
                            normalized.clone(),
                        ));
                    }
                }
            }
        }

        // W23: three quotes in a row — either ``,`` or `` `,` pattern
        if !is_suppressed(WARNING_23, s, fs) && config.warning_23_enabled {
            for pos in 0..normalized.len().saturating_sub(2) {
                let b = normalized[pos];
                if (b == b'`' || b == b'\'') && normalized[pos + 1] == b && normalized[pos + 2] == b
                {
                    diagnostics.push(Diagnostic::new(
                        23,
                        DiagnosticKind::Message,
                        file,
                        line_no,
                        pos,
                        3,
                        "Either `\\,`` or ``\\,` will look better.",
                        normalized.clone(),
                    ));
                    break; // one per line
                }
            }
        }

        // W39: space before ~ → emDblSpace (upstream: case '~': if space before → PSERR)
        if !is_suppressed(WARNING_39, s, fs) && config.warning_39_enabled {
            for (pos, &b) in normalized.iter().enumerate() {
                if b == b'~' && pos > 0 && normalized[pos - 1] == b' ' {
                    diagnostics.push(Diagnostic::new(
                        WARNING_39,
                        DiagnosticKind::Message,
                        file,
                        line_no,
                        pos - 1,
                        1,
                        "Double space found.",
                        normalized.clone(),
                    ));
                }
            }
        }

        // W18: fires on `"` (matching upstream: case '"': HERE(1, emUseQuoteLiga))
        if !is_suppressed(WARNING_18, s, fs) && config.warning_18_enabled {
            for (pos, &b) in normalized.iter().enumerate() {
                if b == b'"'
                    && (pos == 0 || !matches!(normalized[pos - 1], b'\\' | b'`' | b'\''))
                    && !matches!(normalized.get(pos + 1), Some(b'`' | b'\''))
                {
                    let in_comment = tokens.iter().any(|t| {
                        matches!(t.kind, TokenKind::Comment(_))
                            && pos >= t.span.start
                            && pos < t.span.end
                    });
                    if !in_comment {
                        diagnostics.push(Diagnostic::new(
                            WARNING_18,
                            DiagnosticKind::Message,
                            file,
                            line_no,
                            pos,
                            1,
                            "Use either `` or '' as an alternative to `\"'.",
                            normalized.clone(),
                        ));
                    }
                }
            }
        }
        // W19: fires on `\xB4` (Latin-1 acute accent)
        if !is_suppressed(WARNING_19, s, fs) && config.warning_19_enabled {
            for (pos, &b) in normalized.iter().enumerate() {
                if b == 0xB4 {
                    let in_comment = tokens.iter().any(|t| {
                        matches!(t.kind, TokenKind::Comment(_))
                            && pos >= t.span.start
                            && pos < t.span.end
                    });
                    if !in_comment {
                        diagnostics.push(Diagnostic::new(
                            WARNING_19,
                            DiagnosticKind::Message,
                            file,
                            line_no,
                            pos,
                            1,
                            "Use \"'\" (ASCII 39) instead  of \"�\" (ASCII 180).",
                            normalized.clone(),
                        ));
                    }
                }
            }
        }

        if !is_suppressed(WARNING_21, s, fs) {
            for mut diag in warning_21_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if config.warning_22_enabled && !state.file_suppressions.contains(&(WARNING_22 as i64)) {
            for mut diag in warning_22_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_12, s, fs) || !is_suppressed(WARNING_13, s, fs) {
            for mut diag in warning_12_13_check(&tokens, config, initial_math, state.french_spacing)
            {
                let n = diag.number;
                if (n == WARNING_12 && !is_suppressed(WARNING_12, s, fs))
                    || (n == WARNING_13 && !is_suppressed(WARNING_13, s, fs))
                {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
        }

        if !is_suppressed(WARNING_24, s, fs) {
            for mut diag in warning_24_check(&tokens, config) {
                if continued_from_inline_comment {
                    continue;
                }
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_25, s, fs) {
            for mut diag in warning_25_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_26, s, fs) {
            for mut diag in warning_26_check(&tokens, config, initial_math) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_14, s, fs) {
            for mut diag in warning_14_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_27, s, fs) {
            for mut diag in warning_27_check(&tokens, config, file) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_29, s, fs) {
            for mut diag in warning_29_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_30, s, fs) {
            // Pass normalized length minus 1 to exclude the appended trailing space
            for mut diag in warning_30_check(
                &tokens,
                config,
                normalized.len() - 1,
                initial_math,
                !state.seen_space,
            ) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }
        // Update SeenSpace: true if this line had any space in its content
        // (excluding leading spaces at position 0 and trailing auto-spaces)
        let real_line_end = normalized.len().saturating_sub(1);
        for token in tokens.iter().rev() {
            if token.span.end <= real_line_end {
                state.seen_space = matches!(token.kind, TokenKind::Space);
                break;
            }
        }

        if !is_suppressed(WARNING_32, s, fs)
            || !is_suppressed(WARNING_33, s, fs)
            || !is_suppressed(WARNING_34, s, fs)
            || !is_suppressed(WARNING_38, s, fs)
        {
            for mut diag in warning_32_33_34_38_check(&tokens, config, initial_math) {
                let n = diag.number;
                if (n == WARNING_32 && !is_suppressed(WARNING_32, s, fs))
                    || (n == WARNING_33 && !is_suppressed(WARNING_33, s, fs))
                {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
            for mut diag in warning_34_line_check(&normalized, config) {
                if !is_suppressed(WARNING_34, s, fs) {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
            for mut diag in warning_38_line_check(&normalized, config) {
                if !is_suppressed(WARNING_38, s, fs) {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
        }

        if !is_suppressed(WARNING_35, s, fs) {
            for mut diag in warning_35_check(&tokens, config, &math_modes) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_36, s, fs) || !is_suppressed(WARNING_37, s, fs) {
            for mut diag in warning_36_37_check(&tokens, config, &math_modes) {
                if is_in_beamer_overlay(&normalized, diag.column) {
                    continue;
                }
                let n = diag.number;
                if (n == WARNING_36 && !is_suppressed(WARNING_36, s, fs))
                    || (n == WARNING_37 && !is_suppressed(WARNING_37, s, fs))
                {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
        }

        if !is_suppressed(WARNING_39, s, fs) {
            // For now, warning 39 is similar to warning 2 (missing ~)
        }

        if !is_suppressed(WARNING_41, s, fs) {
            for mut diag in warning_41_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_42, s, fs) {
            for mut diag in warning_42_check(&tokens, config) {
                if continued_from_inline_comment {
                    continue;
                }
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_43, s, fs) {
            for mut diag in warning_43_check(&tokens, config) {
                diag.file = file.to_string();
                diag.line = line_no;
                diag.source = normalized.clone();
                diagnostics.push(diag);
            }
        }

        if !is_suppressed(WARNING_45, s, fs) || !is_suppressed(WARNING_46, s, fs) {
            for mut diag in warning_45_46_check(&tokens, config, &math_modes, normalized.len()) {
                if (diag.number == WARNING_45 && !is_suppressed(WARNING_45, s, fs))
                    || (diag.number == WARNING_46 && !is_suppressed(WARNING_46, s, fs))
                {
                    diag.file = file.to_string();
                    diag.line = line_no;
                    diag.source = normalized.clone();
                    diagnostics.push(diag);
                }
            }
        }

        suppress_header_diagnostics(
            config,
            line_started_in_header,
            line_diagnostics_start,
            &mut diagnostics,
        );

        if let Some(file_supps) = pending_file_suppressions {
            state.file_suppressions = file_supps;
        }
    }

    // W16: math mode still on at end of file
    if config.warning_16_enabled && (state.math_mode || state.display_math) {
        let mut diagnostic = Diagnostic::new(
            16,
            DiagnosticKind::Message,
            if state.command_math_mode { "" } else { file },
            eof_line,
            0,
            0,
            "Mathmode still on at end of LaTeX file.",
            Vec::new(),
        );
        diagnostic.sort_line = Some(0);
        diagnostics.push(diagnostic);
    }

    // W17: bracket count mismatch (emNoMatchCC)
    {
        let mut open_brace = 0usize;
        let mut close_brace = 0usize;
        let mut open_bracket = 0usize;
        let mut close_bracket = 0usize;
        let mut open_paren = 0usize;
        let mut close_paren = 0usize;
        for line in split_lines_preserve(input) {
            let normalized = normalize_input_line(line);
            for token in lex_line(&normalized) {
                match token.kind {
                    TokenKind::BeginGroup => open_brace += 1,
                    TokenKind::EndGroup => close_brace += 1,
                    TokenKind::BeginOptional => open_bracket += 1,
                    TokenKind::EndOptional => close_bracket += 1,
                    TokenKind::Punctuation(b'(') => open_paren += 1,
                    TokenKind::Punctuation(b')') => close_paren += 1,
                    _ => {}
                }
            }
        }
        if open_brace != close_brace {
            let (e, f) = ('{', '}');
            if !(state.math_mode || state.display_math) {
                push_unmatched_delimiter_diagnostic(&mut diagnostics, config, file, input, e as u8);
            }
            if config.warning_17_enabled {
                let mut diagnostic = Diagnostic::new(
                    17,
                    DiagnosticKind::Message,
                    file,
                    eof_line,
                    0,
                    0,
                    format!("Number of `{}' doesn't match the number of `{}'!", e, f),
                    Vec::new(),
                );
                diagnostic.sort_line = Some(0);
                diagnostics.push(diagnostic);
            }
        }
        if open_bracket != close_bracket {
            let (e, f) = ('[', ']');
            if !(state.math_mode || state.display_math) {
                push_unmatched_delimiter_diagnostic(&mut diagnostics, config, file, input, e as u8);
            }
            if config.warning_17_enabled {
                let mut diagnostic = Diagnostic::new(
                    17,
                    DiagnosticKind::Message,
                    file,
                    eof_line,
                    0,
                    0,
                    format!("Number of `{}' doesn't match the number of `{}'!", e, f),
                    Vec::new(),
                );
                diagnostic.sort_line = Some(0);
                diagnostics.push(diagnostic);
            }
        }
        if open_paren != close_paren {
            let (e, f) = ('(', ')');
            if !(state.math_mode || state.display_math) {
                push_unmatched_delimiter_diagnostic(&mut diagnostics, config, file, input, e as u8);
            }
            if config.warning_17_enabled {
                let mut diagnostic = Diagnostic::new(
                    17,
                    DiagnosticKind::Message,
                    file,
                    eof_line,
                    0,
                    0,
                    format!("Number of `{}' doesn't match the number of `{}'!", e, f),
                    Vec::new(),
                );
                diagnostic.sort_line = Some(0);
                diagnostics.push(diagnostic);
            }
        }
    }

    // W15: unmatched LaTeX environments
    if config.warning_15_enabled && !(state.math_mode || state.display_math) {
        for frame in state.environment_stack.iter().rev() {
            let mut diagnostic = Diagnostic::new(
                15,
                DiagnosticKind::Warning,
                file,
                frame.line,
                frame.column,
                frame.len,
                format!("No match found for `{}'.", frame.name),
                frame.source.clone(),
            );
            diagnostic.sort_line = Some(0);
            diagnostics.push(diagnostic);
        }
    }

    // W48: unmatched ConTeXt starts
    if config.warning_48_enabled {
        for frame in &state.context_stack {
            let mut diagnostic = Diagnostic::new(
                48,
                DiagnosticKind::Message,
                file,
                frame.line,
                frame.column,
                frame.name.len(),
                format!("No match found for `{}' (ConTeXt).", frame.name),
                frame.source.clone(),
            );
            diagnostic.sort_line = Some(0);
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

// ====== check_line (backward compatible) ======

fn push_unmatched_delimiter_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    config: &CheckerConfig,
    file: &str,
    input: &[u8],
    delimiter: u8,
) {
    if !config.warning_15_enabled {
        return;
    }
    let Some((line_no, column, source)) = find_first_delimiter(input, delimiter) else {
        return;
    };
    let mut diagnostic = Diagnostic::new(
        15,
        DiagnosticKind::Warning,
        file,
        line_no,
        column,
        1,
        format!("No match found for `{}'.", delimiter as char),
        source,
    );
    diagnostic.sort_line = Some(0);
    diagnostics.push(diagnostic);
}

fn find_first_delimiter(input: &[u8], delimiter: u8) -> Option<(i64, usize, Vec<u8>)> {
    for (line_index, line) in split_lines_preserve(input).enumerate() {
        let normalized = normalize_input_line(line);
        if let Some(column) = normalized.iter().position(|byte| *byte == delimiter) {
            let line_no = i64::try_from(line_index + 1).unwrap_or(i64::MAX);
            return Some((line_no, column, normalized));
        }
    }
    None
}

pub fn check_line(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let tokens = lex_line(line);
    let mut diagnostics = Vec::new();
    let mut math_mode = false;

    if config.user_warn_enabled {
        diagnostics.extend(check_user_warn_line(file, line_no, line, config));
    }
    #[cfg(feature = "regex-bytes")]
    if config.user_warn_regex_enabled {
        diagnostics.extend(check_user_warn_regex_line(file, line_no, line, config));
    }

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::MathShift { .. }) {
            math_mode = !math_mode;
            continue;
        }

        if !config.warning_1_enabled || math_mode {
            continue;
        }

        let TokenKind::Command(command) = &token.kind else {
            continue;
        };

        if command.len() == 2
            || config.silent_commands.contains(command)
            || command.starts_with(br"\verb")
        {
            continue;
        }

        if let Some(space) = next_space_token(&tokens, index) {
            diagnostics.push(Diagnostic::new(
                WARNING_1,
                DiagnosticKind::Warning,
                file,
                line_no,
                space.span.start,
                1,
                "Command terminated with space.",
                line.to_vec(),
            ));
        }
    }

    diagnostics
}

// ====== User warning helpers ======

#[cfg(feature = "regex-bytes")]
fn check_user_warn_regex_line(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for user_regex in &config.user_warn_regex {
        #[cfg(feature = "fancy-regex")]
        if let Some(ref fancy) = user_regex.fancy {
            if let Ok(Some(found)) = FancyRegexEngine::find_from(fancy, line, 0) {
                let message = match &user_regex.display {
                    Some(display) if !display.is_empty() => format!("User Regex: {display}."),
                    _ => format!(
                        "User Regex: {}.",
                        String::from_utf8_lossy(&line[found.start..found.end])
                    ),
                };
                diagnostics.push(Diagnostic::new(
                    WARNING_44,
                    DiagnosticKind::Warning,
                    file,
                    line_no,
                    found.start,
                    found.len(),
                    message,
                    line.to_vec(),
                ));
            }
            continue;
        }
        let Ok(matches) = BytesRegexEngine::find_iter(&user_regex.regex, line) else {
            continue;
        };
        for found in matches {
            let message = match &user_regex.display {
                Some(display) if !display.is_empty() => format!("User Regex: {display}."),
                _ => format!(
                    "User Regex: {}.",
                    String::from_utf8_lossy(&line[found.start..found.end])
                ),
            };

            diagnostics.push(Diagnostic::new(
                WARNING_44,
                DiagnosticKind::Warning,
                file,
                line_no,
                found.start,
                found.len(),
                message,
                line.to_vec(),
            ));
        }
    }

    diagnostics
}

fn check_user_warn_line(
    file: &str,
    line_no: i64,
    line: &[u8],
    config: &CheckerConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for pattern in &config.user_warn {
        diagnostics.extend(find_plain_pattern(line, pattern).map(|column| {
            user_warn_diagnostic(file, line_no, line, column, pattern.len(), pattern)
        }));
    }

    if !config.user_warn_case_insensitive.is_empty() {
        let folded_line = ascii_lowercase(line);
        for pattern in &config.user_warn_case_insensitive {
            let folded_pattern = ascii_lowercase(pattern);
            diagnostics.extend(
                find_plain_pattern(&folded_line, &folded_pattern).map(|column| {
                    user_warn_diagnostic(file, line_no, line, column, pattern.len(), pattern)
                }),
            );
        }
    }

    diagnostics
}

fn user_warn_diagnostic(
    file: &str,
    line_no: i64,
    line: &[u8],
    column: usize,
    len: usize,
    pattern: &[u8],
) -> Diagnostic {
    Diagnostic::new(
        WARNING_20,
        DiagnosticKind::Warning,
        file,
        line_no,
        column,
        len,
        format!(
            "User-specified pattern found: {}.",
            String::from_utf8_lossy(pattern)
        ),
        line.to_vec(),
    )
}

// ====== Utility functions ======

fn find_plain_pattern<'a>(
    haystack: &'a [u8],
    needle: &'a [u8],
) -> impl Iterator<Item = usize> + 'a {
    let mut offset = 0;
    std::iter::from_fn(move || {
        if needle.is_empty() || offset > haystack.len() {
            return None;
        }
        let found = haystack[offset..]
            .windows(needle.len())
            .position(|window| window == needle)?;
        let start = offset + found;
        offset = start + needle.len();
        Some(start)
    })
}

fn ascii_lowercase(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte.to_ascii_lowercase()).collect()
}

fn next_space_token(tokens: &[Token], index: usize) -> Option<&Token> {
    tokens
        .get(index + 1)
        .filter(|token| matches!(token.kind, TokenKind::Space))
}

fn split_lines_preserve(input: &[u8]) -> impl Iterator<Item = &[u8]> {
    input.split_inclusive(|byte| *byte == b'\n')
}

fn last_input_line(input: &[u8]) -> i64 {
    let lines = split_lines_preserve(input).count();
    i64::try_from(lines.max(1)).unwrap_or(i64::MAX)
}

fn normalize_input_line(line: &[u8]) -> Vec<u8> {
    let mut normalized = line
        .iter()
        .map(|byte| match byte {
            b'\n' | b'\r' => b' ',
            other => *other,
        })
        .collect::<Vec<_>>();

    // Always append a trailing space, matching upstream ChkTeX behavior.
    normalized.push(b' ');

    normalized
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        checker::{
            CheckerConfig, WARNING_1, WARNING_11, WARNING_20, WARNING_22, WARNING_25, WARNING_26,
            WARNING_30, WARNING_35, WARNING_36, WARNING_41, WARNING_44, WARNING_45, WARNING_46,
            check_document, check_line, parse_file_suppressions, parse_line_suppressions,
        },
        diagnostic::{FormatOptions, format_diagnostic},
        resource::parse_resource,
    };

    // Existing tests kept unchanged
    #[test]
    fn warning_1_reports_command_terminated_by_space() {
        let diagnostics = check_line(
            "stdin",
            1,
            br"\foo This is an error.  ",
            &CheckerConfig::default(),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, WARNING_1);
        assert_eq!(diagnostics[0].column, 4);
        assert_eq!(diagnostics[0].len, 1);
        assert_eq!(
            format_diagnostic(&diagnostics[0], &FormatOptions::normal()),
            "Warning 1 in stdin line 1: Command terminated with space.\n\\foo This is an error.  \n    ^\n"
        );
    }

    #[test]
    fn warning_1_ignores_single_character_control_sequences() {
        let diagnostics = check_line("stdin", 1, br"\{ text", &CheckerConfig::default());

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn warning_1_ignores_silent_commands_from_resources() {
        let resources = parse_resource(r"Silent { \foo }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, br"\foo text", &config);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn warning_1_ignores_commands_in_math_mode() {
        let diagnostics = check_line("stdin", 1, br"$\foo x$ \bar y", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].column, 13);
    }

    #[test]
    fn check_document_tracks_line_numbers() {
        let diagnostics = check_document("file.tex", b"ok\n\\foo bad\n", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(diagnostics[0].file, "file.tex");
        assert_eq!(diagnostics[0].source, br"\foo bad  ".to_vec());
    }

    #[test]
    fn check_document_appends_space_to_final_line_without_newline() {
        let diagnostics = check_document("file.tex", br"\foo bad", &CheckerConfig::default());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source, br"\foo bad ".to_vec());
    }

    #[test]
    fn warning_1_can_be_disabled() {
        let config = CheckerConfig {
            warning_1_enabled: false,
            user_warn_enabled: true,
            user_warn_regex_enabled: true,
            silent_commands: BTreeSet::new(),
            user_warn: Vec::new(),
            user_warn_case_insensitive: Vec::new(),
            #[cfg(feature = "regex-bytes")]
            user_warn_regex: Vec::new(),
            ..Default::default()
        };

        assert!(check_line("stdin", 1, br"\foo text", &config).is_empty());
    }

    #[test]
    fn user_warn_reports_case_sensitive_patterns() {
        let resources = parse_resource(r"UserWarn { TODO }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"TODO and TODO ", &config);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].number, WARNING_20);
        assert_eq!(diagnostics[0].column, 0);
        assert_eq!(diagnostics[1].column, 9);
        assert_eq!(
            format_diagnostic(&diagnostics[0], &FormatOptions::normal()),
            "Warning 20 in stdin line 1: User-specified pattern found: TODO.\nTODO and TODO \n^^^^\n"
        );
    }

    #[test]
    fn user_warn_reports_case_insensitive_patterns() {
        let resources = parse_resource(r"UserWarn [ chktex ]").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"ChkTeX ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].column, 0);
        assert_eq!(diagnostics[0].len, 6);
        assert_eq!(
            diagnostics[0].message,
            "User-specified pattern found: chktex."
        );
    }

    #[test]
    fn user_warn_can_be_disabled() {
        let resources = parse_resource(r"UserWarn { TODO }").unwrap();
        let mut config = CheckerConfig::from_resources(&resources);
        config.user_warn_enabled = false;

        assert!(check_line("stdin", 1, b"TODO ", &config).is_empty());
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_reports_default_match_text() {
        let resources = parse_resource(r"UserWarnRegex { intro }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, b"a good intro ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, WARNING_44);
        assert_eq!(diagnostics[0].column, 7);
        assert_eq!(diagnostics[0].message, "User Regex: intro.");
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_reports_display_comment() {
        let resources =
            parse_resource(r"UserWarnRegex { (?!#Always! use! \nmid)\\not! *(\\mid|\|) }").unwrap();
        let config = CheckerConfig::from_resources(&resources);
        let diagnostics = check_line("stdin", 1, br"$p\not|n$ ", &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].number, WARNING_44);
        assert_eq!(diagnostics[0].message, r"User Regex: Always use \nmid.");
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_skips_pcre_prefixed_patterns_in_default_engine() {
        let resources = parse_resource(r"UserWarnRegex { PCRE:\[(?!bad) }").unwrap();
        let config = CheckerConfig::from_resources(&resources);

        assert!(config.user_warn_regex.is_empty());
    }

    #[cfg(feature = "regex-bytes")]
    #[test]
    fn user_warn_regex_can_be_disabled() {
        let resources = parse_resource(r"UserWarnRegex { intro }").unwrap();
        let mut config = CheckerConfig::from_resources(&resources);
        config.user_warn_regex_enabled = false;

        assert!(check_line("stdin", 1, b"intro ", &config).is_empty());
    }

    // ====== New infrastructure tests ======

    #[test]
    fn parses_line_suppressions_single() {
        let s = parse_line_suppressions(b"text % chktex 22");
        assert_eq!(s.numbers, vec![22]);
    }

    #[test]
    fn parses_line_suppressions_multiple() {
        let s = parse_line_suppressions(b"text % chktex 35 36");
        assert_eq!(s.numbers, vec![35, 36]);
    }

    #[test]
    fn parses_line_suppressions_all() {
        let s = parse_line_suppressions(b"text % chktex -1");
        assert_eq!(s.numbers, vec![-1]);
    }

    #[test]
    fn parses_line_suppressions_are_case_sensitive() {
        let s = parse_line_suppressions(b"text % CHKTEX 22");
        assert!(s.numbers.is_empty());
    }

    #[test]
    fn parses_line_suppressions_repeated_markers() {
        let s = parse_line_suppressions(b"text % chktex 36 chktex 35");
        assert_eq!(s.numbers, vec![36, 35]);
    }

    #[test]
    fn parses_file_suppressions() {
        let s = parse_file_suppressions(b"% chktex-file 22").unwrap();
        assert!(s.contains(&22));
    }

    #[test]
    fn parses_file_suppressions_ignores_line_suppressions() {
        let s = parse_file_suppressions(b"text % chktex 22");
        assert!(s.is_none());
    }

    // ====== New warning tests ======

    fn make_config_with_resources(src: &str) -> CheckerConfig {
        parse_resource(src)
            .map(|r| CheckerConfig::from_resources(&r))
            .unwrap()
    }

    #[test]
    fn warning_22_detects_comment() {
        let mut config = CheckerConfig::default();
        config.set_warning_enabled(WARNING_22, true);
        let d = check_document("test.tex", b"text % comment\n", &config);
        let w22: Vec<_> = d.iter().filter(|d| d.number == WARNING_22).collect();
        assert!(!w22.is_empty(), "should find warning 22");
        assert!(w22[0].column > 0);
    }

    #[test]
    fn line_suppression_does_not_affect_other_lines() {
        let mut config = CheckerConfig::default();
        config.set_warning_enabled(WARNING_22, true);
        // Line 1 suppresses warning 22, line 2 should still get it
        let d = check_document("test.tex", b"% chktex 22\n% comment\n", &config);
        let w22: Vec<_> = d.iter().filter(|d| d.number == WARNING_22).collect();
        assert_eq!(w22.len(), 2, "line suppressions should not hide warning 22");
        assert_eq!(w22[0].line, 1);
        assert_eq!(w22[1].line, 2);
    }

    #[test]
    fn no_line_suppression_disables_chktex_comments() {
        let mut config = CheckerConfig::default();
        let suppressed = check_document("test.tex", b"Here(warn) % chktex 36\n", &config);
        assert!(!suppressed.iter().any(|diag| diag.number == WARNING_36));

        config.no_line_suppression = true;
        let unsuppressed = check_document("test.tex", b"Here(warn) % chktex 36\n", &config);
        assert!(unsuppressed.iter().any(|diag| diag.number == WARNING_36));
    }

    #[test]
    fn warning_30_detects_multiple_spaces() {
        let mut config = CheckerConfig::default();
        config.set_warning_enabled(WARNING_30, true);
        let d = check_document("test.tex", b"double  space\n", &config);
        let w30: Vec<_> = d.iter().filter(|d| d.number == WARNING_30).collect();
        assert!(!w30.is_empty(), "should find warning 30");
    }

    #[test]
    fn warning_11_detects_ellipsis() {
        let config = CheckerConfig::default();
        let d = check_document("test.tex", b"Foo...bar\n", &config);
        let w11: Vec<_> = d.iter().filter(|d| d.number == WARNING_11).collect();
        assert!(!w11.is_empty(), "should find warning 11");
    }

    #[test]
    fn warning_25_detects_missing_braces() {
        let config = CheckerConfig::default();
        let d = check_document("test.tex", b"10^10\n", &config);
        let w25: Vec<_> = d.iter().filter(|d| d.number == WARNING_25).collect();
        assert!(!w25.is_empty(), "should find warning 25");
    }

    #[test]
    fn warning_26_detects_space_before_question() {
        let config = CheckerConfig::default();
        let d = check_document("test.tex", b"Do you understand ?\n", &config);
        let w26: Vec<_> = d.iter().filter(|d| d.number == WARNING_26).collect();
        assert!(!w26.is_empty(), "should find warning 26");
    }

    #[test]
    fn warning_35_detects_math_operators() {
        let config = make_config_with_resources(r"MathRoman { sin cos }");
        let d = check_document("test.tex", b"$sin^2 + cos^2 = 1$\n", &config);
        let w35: Vec<_> = d.iter().filter(|d| d.number == WARNING_35).collect();
        assert_eq!(w35.len(), 2, "should find sin and cos as warnings");
    }

    #[test]
    fn warning_41_detects_primitives() {
        let mut config = make_config_with_resources(r"Primitives { \above }");
        config.set_warning_enabled(WARNING_41, true);
        let d = check_document("test.tex", br"foo \above qux\n", &config);
        // Note: warning_1_check will fire first for \above
        let w41: Vec<_> = d.iter().filter(|d| d.number == WARNING_41).collect();
        assert!(!w41.is_empty(), "should find warning 41 for \\above");
    }

    #[test]
    fn warning_45_detects_display_math() {
        let config = CheckerConfig::default();
        let d = check_document("test.tex", b"$$\n", &config);
        let w45: Vec<_> = d.iter().filter(|d| d.number == WARNING_45).collect();
        assert!(!w45.is_empty(), "should find warning 45 for $$");
    }

    #[test]
    fn warning_46_detects_inline_math() {
        let mut config = CheckerConfig::default();
        config.set_warning_enabled(WARNING_46, true);
        let d = check_document("test.tex", b"$\n", &config);
        let w46: Vec<_> = d.iter().filter(|d| d.number == WARNING_46).collect();
        assert!(!w46.is_empty(), "should find warning 46 for $");
    }
}
