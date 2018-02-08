use super::{Record, RecordKind};
use std::fmt::{Display, Formatter, Result};

impl Display for RecordKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Record::*;

        let kind = self.kind();
        match *self {
            TestName { ref name } => write!(f, "{}:{}", kind, name)?,
            SourceFile { ref path } => write!(f, "{}:{}", kind, path.display())?,
            FunctionName {
                ref name,
                start_line,
            } => write!(f, "{}:{},{}", kind, start_line, name)?,
            FunctionData { ref name, count } => write!(f, "{}:{},{}", kind, count, name)?,
            FunctionsFound { found } | BranchesFound { found } | LinesFound { found } => {
                write!(f, "{}:{}", kind, found)?
            }
            FunctionsHit { hit } | BranchesHit { hit } | LinesHit { hit } => {
                write!(f, "{}:{}", kind, hit)?
            }
            BranchData {
                line,
                block,
                branch,
                taken: Some(taken),
            } => write!(f, "{}:{},{},{},{}", kind, line, block, branch, taken)?,
            BranchData {
                line,
                block,
                branch,
                taken: None,
            } => write!(f, "{}:{},{},{},-", kind, line, block, branch)?,
            LineData {
                line,
                count,
                checksum: Some(ref checksum),
            } => write!(f, "{}:{},{},{}", kind, line, count, checksum)?,
            LineData {
                line,
                count,
                checksum: None,
            } => write!(f, "{}:{},{}", kind, line, count)?,
            EndOfRecord => write!(f, "{}", kind)?,
        }
        Ok(())
    }
}
