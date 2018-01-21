use super::Record;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Fail, Eq, PartialEq)]
pub enum Error {
    #[fail(display = "field `{}` not found", _0)] FieldNotFound(&'static str),
    #[fail(display = "too many fields found")] TooManyFields,
    #[fail(display = "invalid value of field `{}`: {}", _0, _1)]
    ParseIntError(&'static str, #[cause] ParseIntError),
    #[fail(display = "unknown record")] UnknownRecord,
}

macro_rules! replace_expr {
    ($_id:ident $sub:expr) => {$sub}
}
macro_rules! count_idents {
    ($($id:ident)*) => { 0 $(+ replace_expr!($id 1))* }
}
macro_rules! parse_record {
    ($input:expr => $rec:ident { $($field:ident,)* .. $last: ident}) => {{
        let mut sp = $input.splitn(count_idents!($($field)*) + 1, ',');
        let rec = $rec {
            $($field: ParseField::parse_iter_next(&mut sp, stringify!($field))?,)*
            $last: ParseField::parse_iter_next(&mut sp, stringify!($last))?
        };
        debug_assert!(sp.next().is_none());
        Ok(rec)
    }};
    ($input:expr => $rec:ident { $($field:ident,)* .. ?$last: ident}) => {{
        let mut sp = $input.splitn(count_idents!($($field)*) + 1, ',');
        let rec = $rec {
            $($field: ParseField::parse_iter_next(&mut sp, stringify!($field))?,)*
            $last: if let Some(s) = sp.next() {
                ParseField::parse_field(s, stringify!($last))?
            } else {
                None
            }
        };
        debug_assert!(sp.next().is_none());
        Ok(rec)
    }};
    ($input:expr => $rec:ident { $($field:ident),* $(,?$opt_field:ident),* }) => {{
        let mut sp = $input.split(',');
        let rec = $rec {
            $($field: ParseField::parse_iter_next(&mut sp, stringify!($field))?,)*
            $($opt_field: if let Some(s) = sp.next() {
                Some(ParseField::parse_field(s, stringify!($opt_field))?)
            } else {
                None
            },)*
        };
        if sp.next().is_some() {
            return Err(Error::TooManyFields)
        }
        Ok(rec)
    }};
}

impl FromStr for Record {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        use Record::*;

        s = s.trim_right_matches(&['\n', '\r'] as &[_]);
        if s == "end_of_record" {
            return Ok(EndOfRecord);
        }

        let mut sp = s.splitn(2, ":");
        let kind = sp.next().unwrap();
        let body = sp.next().unwrap_or("");
        debug_assert!(sp.next().is_none());

        match kind {
            "TN" => parse_record!(body => TestName { .. name }),
            "SF" => parse_record!(body => SourceFile { .. path }),
            "FN" => parse_record!(body => FunctionName { start_line, .. name }),
            "FNDA" => parse_record!(body => FunctionData { count, .. name }),
            "FNF" => parse_record!(body => FunctionsFound { found }),
            "FNH" => parse_record!(body => FunctionsHit { hit }),
            "BRDA" => parse_record!(body => BranchData { line, block, branch, taken}),
            "BRF" => parse_record!(body => BranchesFound { found }),
            "BRH" => parse_record!(body => BranchesHit { hit }),
            "DA" => parse_record!(body => LineData { line, count, .. ?checksum }),
            "LH" => parse_record!(body => LinesHit { hit }),
            "LF" => parse_record!(body => LinesFound { found }),
            _ => Err(Error::UnknownRecord),
        }
    }
}

trait ParseField: Sized {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, Error>;
    fn parse_iter_next<'a, I>(it: &mut I, name: &'static str) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a str>,
    {
        let s = it.next().ok_or(Error::FieldNotFound(name))?;
        Self::parse_field(s, name)
    }
}

impl ParseField for String {
    fn parse_field(s: &str, _name: &'static str) -> Result<Self, Error> {
        Ok(s.into())
    }
}

impl ParseField for PathBuf {
    fn parse_field(s: &str, _name: &'static str) -> Result<Self, Error> {
        Ok(From::from(s))
    }
}

impl ParseField for u32 {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, Error> {
        s.parse().map_err(|e| Error::ParseIntError(name, e))
    }
}

impl ParseField for u64 {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, Error> {
        s.parse().map_err(|e| Error::ParseIntError(name, e))
    }
}
impl<T> ParseField for Option<T>
where
    T: ParseField,
{
    fn parse_field(s: &str, name: &'static str) -> Result<Self, Error> {
        let val = if s == "-" {
            None
        } else {
            Some(ParseField::parse_field(s, name)?)
        };
        Ok(val)
    }
}
