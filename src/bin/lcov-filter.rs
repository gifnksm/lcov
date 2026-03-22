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

    /// Remap source file path using regex, format: REGEX=REPLACEMENT (can repeat).
    /// Supports capture groups: $1, $name, \1, \g<name>
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
        let replacement = normalize_replacement(rep);
        Ok(RegexMap {
            pattern,
            replacement,
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
            s = m
                .pattern
                .replace_all(&s, m.replacement.as_str())
                .into_owned();
        }
        current = PathBuf::from(s);
    }

    current
}

// Normalizes a user-provided regex replacement to Rust `regex` style.
// Converts common group reference syntaxes into `$...` so the `regex` crate
// can interpret them:
// - `\1`, `\2`, ...  => `$1`, `$2`
// - `\g<name>`       => `$name`
// - already valid `$1`, `$name` are left unchanged
// This enables `--remap-path-regex` to accept PCRE/Python-style replacements
// without the user having to rewrite them. Characters not part of an escape
// are passed through verbatim.
fn normalize_replacement(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.peek().copied() {
                Some(d) if d.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(d2) = it.peek().copied() {
                        if d2.is_ascii_digit() {
                            num.push(d2);
                            let _ = it.next();
                        } else {
                            break;
                        }
                    }
                    out.push('$');
                    out.push_str(&num);
                    continue;
                }
                Some('g') => {
                    let _ = it.next();
                    if it.peek() == Some(&'<') {
                        let _ = it.next();
                        let mut name = String::new();
                        while let Some(ch) = it.peek().copied() {
                            if ch == '>' {
                                let _ = it.next();
                                break;
                            }
                            name.push(ch);
                            let _ = it.next();
                        }
                        out.push('$');
                        out.push_str(&name);
                        continue;
                    } else {
                        out.push('\\');
                        out.push('g');
                        continue;
                    }
                }
                _ => {
                    out.push('\\');
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn main() {
    let opt = Opt::parse();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn normalize_replacement_variants() {
        assert_eq!(normalize_replacement("$1"), "$1");
        assert_eq!(normalize_replacement("$name"), "$name");
        assert_eq!(normalize_replacement("\\1"), "$1");
        assert_eq!(normalize_replacement("\\12"), "$12");
        assert_eq!(normalize_replacement("\\g<name>"), "$name");
        assert_eq!(normalize_replacement("\\g<1>"), "$1");
        assert_eq!(normalize_replacement("pre\\g<name>post"), "pre$namepost");
        // Unknown escapes are preserved
        assert_eq!(normalize_replacement("\\x"), "\\x");
    }

    #[test]
    fn remap_path_prefix_basic() {
        let p = Path::new("/home/user/project/src/foo.c");
        let prefixes = vec![PrefixMap {
            from: PathBuf::from("/home/user"),
            to: PathBuf::from("/mnt/user"),
        }];
        let regexes: Vec<RegexMap> = vec![];
        let out = remap_path(p, &prefixes, &regexes);
        assert_eq!(out.to_string_lossy(), "/mnt/user/project/src/foo.c");
    }

    #[test]
    fn remap_path_prefix_then_regex_numeric_groups() {
        let p = Path::new("/home/user/project/src/foo.c");
        let prefixes = vec![PrefixMap {
            from: PathBuf::from("/home/user"),
            to: PathBuf::from("/data/user"),
        }];
        let re = RegexMap::from_str("^/data/(.+)=/src/$1").unwrap();
        let regexes = vec![re];
        let out = remap_path(p, &prefixes, &regexes);
        assert_eq!(out.to_string_lossy(), "/src/user/project/src/foo.c");
    }

    #[test]
    fn remap_path_regex_python_style_named_groups() {
        let p = Path::new("/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c");
        let prefixes: Vec<PrefixMap> = vec![];
        let re = RegexMap::from_str(r"^/home/(?P<user>[^/]+)/(?P<rest>.+)=/mnt/\g<user>/src/$rest")
            .unwrap();
        let regexes = vec![re];
        let out = remap_path(p, &prefixes, &regexes);
        assert_eq!(
            out.to_string_lossy(),
            "/mnt/nksm/src/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c"
        );
    }

    #[test]
    fn remap_path_prefix_windows_style() {
        let p = Path::new("C:\\Users\\alice\\proj\\src\\foo.c");
        let prefixes = vec![PrefixMap {
            from: PathBuf::from("C:\\Users\\alice"),
            to: PathBuf::from("D:\\workspace\\alice"),
        }];
        let regexes: Vec<RegexMap> = vec![];
        let out = remap_path(p, &prefixes, &regexes);
        assert_eq!(
            out.to_string_lossy(),
            "D:\\workspace\\alice\\proj\\src\\foo.c"
        );
    }

    #[test]
    fn remap_path_regex_windows_style_numeric_groups() {
        let p = Path::new("C:\\Users\\alice\\proj\\src\\foo.c");
        let prefixes: Vec<PrefixMap> = vec![];
        let re = RegexMap::from_str(r"^C:\\Users\\([^\\]+)\\(.+)=D:\$1\code\$2").unwrap();
        let regexes = vec![re];
        let out = remap_path(p, &prefixes, &regexes);
        assert_eq!(out.to_string_lossy(), "D:\\alice\\code\\proj\\src\\foo.c");
    }
}
