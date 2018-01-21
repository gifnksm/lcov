use super::Record;
use std::fmt::{Display, Formatter, Result};

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Record::*;

        match self {
            &TestName { ref name } => write!(f, "TN:{}", name)?,
            &SourceFile { ref path } => write!(f, "SF:{}", path.display())?,
            &FunctionName {
                ref name,
                start_line,
            } => write!(f, "FN:{},{}", start_line, name)?,
            &FunctionData { ref name, count } => write!(f, "FNDA:{},{}", count, name)?,
            &FunctionsFound { found } => write!(f, "FNF:{}", found)?,
            &FunctionsHit { hit } => write!(f, "FNH:{}", hit)?,
            &BranchData {
                line,
                block,
                branch,
                taken: Some(taken),
            } => write!(f, "BRDA:{},{},{},{}", line, block, branch, taken)?,
            &BranchData {
                line,
                block,
                branch,
                taken: None,
            } => write!(f, "BRDA:{},{},{},-", line, block, branch)?,
            &BranchesFound { found } => write!(f, "BRF:{}", found)?,
            &BranchesHit { hit } => write!(f, "BRH:{}", hit)?,
            &LineData {
                line,
                count,
                checksum: Some(ref checksum),
            } => write!(f, "DA:{},{},{}", line, count, checksum)?,
            &LineData {
                line,
                count,
                checksum: None,
            } => write!(f, "DA:{},{}", line, count)?,
            &LinesHit { hit } => write!(f, "LH:{}", hit)?,
            &LinesFound { found } => write!(f, "LF:{}", found)?,
            &EndOfRecord => write!(f, "end_of_record")?,
        }
        Ok(())
    }
}
