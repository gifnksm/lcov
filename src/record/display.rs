use super::Record;
use std::fmt::{Display, Formatter, Result};

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Record::*;

        match self {
            &TestName { ref name } => writeln!(f, "TN:{}", name)?,
            &SourceFile { ref path } => writeln!(f, "SF:{}", path.display())?,
            &FunctionName {
                ref name,
                start_line,
            } => writeln!(f, "FN:{},{}", start_line, name)?,
            &FunctionData { ref name, count } => writeln!(f, "FNDA:{},{}", count, name)?,
            &FunctionsFound { found } => writeln!(f, "FNF:{}", found)?,
            &FunctionsHit { hit } => writeln!(f, "FNH:{}", hit)?,
            &BranchData {
                line,
                block,
                branch,
                taken: Some(taken),
            } => writeln!(f, "BRDA:{},{},{},{}", line, block, branch, taken)?,
            &BranchData {
                line,
                block,
                branch,
                taken: None,
            } => writeln!(f, "BRDA:{},{},{},-", line, block, branch)?,
            &BranchesFound { found } => writeln!(f, "BRF:{}", found)?,
            &BranchesHit { hit } => writeln!(f, "BRH:{}", hit)?,
            &LineData {
                line,
                count,
                checksum: Some(ref checksum),
            } => writeln!(f, "DA:{},{},{}", line, count, checksum)?,
            &LineData {
                line,
                count,
                checksum: None,
            } => writeln!(f, "DA:{},{}", line, count)?,
            &LinesHit { hit } => writeln!(f, "LH:{}", hit)?,
            &LinesFound { found } => writeln!(f, "LF:{}", found)?,
            &EndOfRecord => writeln!(f, "end_of_record")?,
        }
        Ok(())
    }
}
