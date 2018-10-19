#[macro_use]
extern crate failure;
extern crate glob;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
extern crate term;
extern crate toml;

mod lint;
mod printer;

use failure::{Error, ResultExt};
use lint::RuleSet;
use printer::Printer;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use structopt::{clap, StructOpt};

// -------------------------------------------------------------------------------------------------
// Usage
// -------------------------------------------------------------------------------------------------

#[derive(Debug, StructOpt)]
#[structopt(name = "flexlint")]
#[structopt(
    raw(long_version = "option_env!(\"LONG_VERSION\").unwrap_or(env!(\"CARGO_PKG_VERSION\"))")
)]
#[structopt(raw(setting = "clap::AppSettings::ColoredHelp"))]
pub struct Opt {
    /// Rule file
    #[structopt(
        short = "r",
        long = "rule",
        default_value = ".flexlint.toml",
        parse(from_os_str)
    )]
    pub rule: PathBuf,

    /// Show results by simple format
    #[structopt(short = "s", long = "simple")]
    pub simple: bool,

    /// Show verbose message
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,
}

// -------------------------------------------------------------------------------------------------
// Main
// -------------------------------------------------------------------------------------------------

pub fn main() {
    let opt = Opt::from_args();
    let exit_code = match run_opt(&opt) {
        Ok(pass) => {
            if pass {
                0
            } else {
                1
            }
        }
        Err(x) => {
            println!("Error: {}", x);
            2
        }
    };

    process::exit(exit_code);
}

pub fn run_opt(opt: &Opt) -> Result<bool, Error> {
    let rule = search_rule(&opt.rule)?;

    let mut f = File::open(&rule)
        .with_context(|_| format!("failed to open: '{}'", rule.to_string_lossy()))?;
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);
    let rule: RuleSet = toml::from_str(&s)
        .with_context(|_| format!("failed to parse toml: '{}'", opt.rule.to_string_lossy()))?;

    let checked = rule.check()?;
    let pass = Printer::print(checked, opt.simple, opt.verbose)?;

    Ok(pass)
}

fn search_rule(rule: &Path) -> Result<PathBuf, Error> {
    let current = env::current_dir()?;
    for dir in current.ancestors() {
        let candidate = dir.join(rule);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format_err!("rule not found: '{}'", rule.to_string_lossy()))
}

// -------------------------------------------------------------------------------------------------
// Test
// -------------------------------------------------------------------------------------------------
