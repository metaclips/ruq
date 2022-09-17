mod args;
mod parser;
mod processor;
mod toml;
mod yaml;

use args::{Args, SupportedLanguages};
use clap::Parser;
use processor::Processor;
use serde_json::Value;
use std::{
    io::{stdin, stdout, Read, Write},
    str::FromStr,
};

fn main() {
    let args = Args::parse();
    let input = match args.input {
        Some(e) => e,
        None => {
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer).unwrap();
            buffer
        }
    };

    let json = match SupportedLanguages::from(args.from.clone()) {
        SupportedLanguages::Json => Value::from_str(input.as_str()).unwrap(),
        SupportedLanguages::Toml => toml::Toml::new(input).to_json(),
        SupportedLanguages::Yaml => yaml::Yaml::new(input).to_json(),
        SupportedLanguages::Unsupported => panic!("Unsupported language"),
    };

    let result = parser::Parser::parse(json, &args.filter);

    let conversion_to = {
        if let Some(to_value) = args.to {
            to_value
        } else {
            args.from
        }
    };

    let result = match SupportedLanguages::from(conversion_to) {
        SupportedLanguages::Json => serde_json::to_string_pretty(&result).unwrap(),
        SupportedLanguages::Toml => toml::Toml::from_json(result).to_string(),
        SupportedLanguages::Yaml => yaml::Yaml::from_json(result).to_string(),
        SupportedLanguages::Unsupported => panic!("Unsupported language"),
    };

    stdout().write_all(result.as_bytes()).unwrap();
}
