use self::branch_list::BranchList;
use self::func_list::FuncList;
use self::line_list::LineList;
use super::{MergeError, Parser, Record};

mod func_list;
mod branch_list;
mod line_list;

/// A coverage information about a source file.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Section {
    func_list: FuncList,
    branch_list: BranchList,
    line_list: LineList,
}

impl Section {
    pub(crate) fn is_empty(&self) -> bool {
        self.func_list.is_empty() && self.branch_list.is_empty() && self.line_list.is_empty()
    }

    pub(crate) fn merge<I, E>(
        &mut self,
        parser: &mut Parser<I, Record>,
    ) -> Result<(), MergeError<E>>
    where
        I: Iterator<Item = Result<Record, E>>,
    {
        self.func_list.merge(parser)?;
        self.branch_list.merge(parser)?;
        self.line_list.merge(parser)?;

        Ok(())
    }

    pub(crate) fn func_list(&mut self) -> &mut FuncList {
        &mut self.func_list
    }
    pub(crate) fn branch_list(&mut self) -> &mut BranchList {
        &mut self.branch_list
    }
    pub(crate) fn line_list(&mut self) -> &mut LineList {
        &mut self.line_list
    }
}

impl IntoIterator for Section {
    type Item = Record;
    type IntoIter = Box<Iterator<Item = Record>>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.func_list
            .into_iter()
            .chain(self.branch_list)
            .chain(self.line_list);
        Box::new(iter)
    }
}
