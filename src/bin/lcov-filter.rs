//! Utility commands to operate and analyze LCOV trace file at blazingly fast.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

use clap::Parser;
use lcov::Report;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;

#[derive(Debug, clap::Parser)]
#[clap(about = "Filters LCOV tracefiles with path remapping")]
struct Opt {
    /// Remap source file path prefix, format: FROM=TO (can repeat)
    #[arg(long = "remap-path-prefix", value_name = "FROM=TO")]
    remap_path_prefix: Vec<PrefixMap>,

    /// Remap source file path using regex, format: REGEX=REPLACEMENT (can repeat)
    #[arg(long = "remap-path-regex", value_name = "REGEX=REPLACEMENT")]
    remap_path_regex: Vec<RegexMap>,

    /// LCOV tracefiles to read and filter
    #[arg(name = "FILE")]
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct PrefixMap {
    from: PathBuf,
    to: PathBuf,
}

impl FromStr for PrefixMap {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from, to) = s
            .split_once('=')
            .ok_or_else(|| "expected FROM=TO".to_string())?;
        if from.is_empty() {
            return Err("FROM must not be empty".into());
        }
        Ok(PrefixMap {
            from: PathBuf::from(from),
            to: PathBuf::from(to),
        })
    }
}

#[derive(Debug, Clone)]
struct RegexMap {
    pattern: Regex,
    replacement: String,
}

impl FromStr for RegexMap {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (pat, rep) = s
            .split_once('=')
            .ok_or_else(|| "expected REGEX=REPLACEMENT".to_string())?;
        let pattern = Regex::new(pat).map_err(|e| e.to_string())?;
        Ok(RegexMap {
            pattern,
            replacement: rep.to_string(),
        })
    }
}

fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let mut report = Report::new();

    for path in &opt.files {
        report.merge(Report::from_file(path)?)?;
    }

    // Stream records and remap only the SourceFile paths.
    for rec in report.into_records() {
        match rec {
            lcov::Record::SourceFile { path } => {
                let new_path = remap_path(&path, &opt.remap_path_prefix, &opt.remap_path_regex);
                let rec2 = lcov::Record::SourceFile { path: new_path };
                println!("{}", rec2);
            }
            _ => println!("{}", rec),
        }
    }

    Ok(())
}

// Note: remapping is applied at record printing time (SF: lines).

fn remap_path(path: &Path, prefixes: &[PrefixMap], regexes: &[RegexMap]) -> PathBuf {
    // First, apply prefix remaps in order.
    let mut current = path.to_path_buf();
    for m in prefixes {
        // Prefer Path::strip_prefix for accurate path boundaries.
        let remapped = if let Ok(rest) = current.strip_prefix(&m.from) {
            m.to.join(rest)
        } else {
            // Fallback to string-based prefix matching for cases where separator styles differ.
            let cur = current.to_string_lossy();
            let from = m.from.to_string_lossy();
            if cur.starts_with(from.as_ref()) {
                let replaced = format!("{}{}", m.to.to_string_lossy(), &cur[from.len()..]);
                PathBuf::from(replaced)
            } else {
                current.clone()
            }
        };
        current = remapped;
    }

    // Then, apply regex remaps in order.
    if !regexes.is_empty() {
        let mut s = current.to_string_lossy().into_owned();
        for m in regexes {
            s = m.pattern.replace_all(&s, m.replacement.as_str()).into_owned();
        }
        current = PathBuf::from(s);
    }

    current
}

fn main() {
    let opt = Opt::parse();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
