use super::{Record, RecordKind};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParseRecordKindError;

impl FromStr for RecordKind {
    type Err = ParseRecordKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RecordKind::*;
        let kind = match s {
            "TN" => TestName,
            "SF" => SourceFile,
            "FN" => FunctionName,
            "FNDA" => FunctionData,
            "FNF" => FunctionsFound,
            "FNH" => FunctionsHit,
            "BRDA" => BranchData,
            "BRF" => BranchesFound,
            "BRH" => BranchesHit,
            "DA" => LineData,
            "LF" => LinesFound,
            "LH" => LinesHit,
            "end_of_record" => EndOfRecord,
            _ => Err(ParseRecordKindError)?,
        };

        Ok(kind)
    }
}

#[derive(Debug, Clone, Fail, Eq, PartialEq)]
pub enum ParseRecordError {
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
            return Err(ParseRecordError::TooManyFields)
        }
        Ok(rec)
    }};
}

impl FromStr for Record {
    type Err = ParseRecordError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        use Record::*;
        use RecordKind as Kind;

        s = s.trim_right_matches::<&[_]>(&['\n', '\r']);
        let mut sp = s.splitn(2, ":");

        let kind = sp.next()
            .unwrap()
            .parse::<RecordKind>()
            .map_err(|_e| ParseRecordError::UnknownRecord)?;
        let body = sp.next().unwrap_or("");
        debug_assert!(sp.next().is_none());

        match kind {
            Kind::TestName => parse_record!(body => TestName { .. name }),
            Kind::SourceFile => parse_record!(body => SourceFile { .. path }),
            Kind::FunctionName => parse_record!(body => FunctionName { start_line, .. name }),
            Kind::FunctionData => parse_record!(body => FunctionData { count, .. name }),
            Kind::FunctionsFound => parse_record!(body => FunctionsFound { found }),
            Kind::FunctionsHit => parse_record!(body => FunctionsHit { hit }),
            Kind::BranchData => parse_record!(body => BranchData { line, block, branch, taken}),
            Kind::BranchesFound => parse_record!(body => BranchesFound { found }),
            Kind::BranchesHit => parse_record!(body => BranchesHit { hit }),
            Kind::LineData => parse_record!(body => LineData { line, count, .. ?checksum }),
            Kind::LinesFound => parse_record!(body => LinesFound { found }),
            Kind::LinesHit => parse_record!(body => LinesHit { hit }),
            Kind::EndOfRecord => Ok(EndOfRecord),
        }
    }
}

trait ParseField: Sized {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, ParseRecordError>;
    fn parse_iter_next<'a, I>(it: &mut I, name: &'static str) -> Result<Self, ParseRecordError>
    where
        I: Iterator<Item = &'a str>,
    {
        let s = it.next().ok_or(ParseRecordError::FieldNotFound(name))?;
        Self::parse_field(s, name)
    }
}

impl ParseField for String {
    fn parse_field(s: &str, _name: &'static str) -> Result<Self, ParseRecordError> {
        Ok(s.into())
    }
}

impl ParseField for PathBuf {
    fn parse_field(s: &str, _name: &'static str) -> Result<Self, ParseRecordError> {
        Ok(From::from(s))
    }
}

impl ParseField for u32 {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, ParseRecordError> {
        s.parse()
            .map_err(|e| ParseRecordError::ParseIntError(name, e))
    }
}

impl ParseField for u64 {
    fn parse_field(s: &str, name: &'static str) -> Result<Self, ParseRecordError> {
        s.parse()
            .map_err(|e| ParseRecordError::ParseIntError(name, e))
    }
}
impl<T> ParseField for Option<T>
where
    T: ParseField,
{
    fn parse_field(s: &str, name: &'static str) -> Result<Self, ParseRecordError> {
        let val = if s == "-" {
            None
        } else {
            Some(ParseField::parse_field(s, name)?)
        };
        Ok(val)
    }
}
