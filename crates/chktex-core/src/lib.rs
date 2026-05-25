pub mod checker;
pub mod cli;
pub mod diagnostic;
pub mod lexer;
pub mod regex_engine;
pub mod resource;

pub const PACKAGE_NAME: &str = "ChkTeX";
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
