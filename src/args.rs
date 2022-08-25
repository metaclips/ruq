use clap::Parser;

/// A lightweight and flexible command-line JSON, TOML processor and converter.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Input, can also be passed as standard input
    #[clap(short, long, value_parser)]
    pub input: Option<String>,

    /// JSON format filter
    #[clap(long, value_parser)]
    pub filter: String,

    /// Object language passed, json, toml, etc.
    #[clap(long, value_parser)]
    pub from: String,

    /// Object language to convert to, JSON, TOML, etc.
    #[clap(long, value_parser)]
    pub to: Option<String>,
}

pub enum SupportedLanguages {
    Json,
    Toml,
    Unsupported,
}

impl From<String> for SupportedLanguages {
    fn from(val: String) -> Self {
        match val.to_lowercase().as_str() {
            "json" => Self::Json,
            "toml" => Self::Toml,
            _ => Self::Unsupported,
        }
    }
}