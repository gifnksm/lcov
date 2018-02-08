macro_rules! eat {
    ($parser:expr, $p:pat) => { eat!($parser, $p => {}) };
    ($parser:expr, $p:pat => $body:expr) => {
        match $parser.pop().map_err(MergeError::Read)? {
            Some($p) => $body,
            Some(rec) => Err(MergeError::UnexpectedRecord(rec.kind()))?,
            None => Err(MergeError::UnexpectedEof)?,
        }
    }
}

macro_rules! eat_if_matches {
    ($parser:expr, $p:pat) => { eat_if_matches!($parser, $p => {}) };
    ($parser:expr, $p:pat => $body:expr) => {
        match $parser.pop().map_err(MergeError::Read)? {
            Some($p) => Some($body),
            Some(item) => {
                $parser.push(item);
                None
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Parser<I, T> {
    iter: I,
    next_item: Option<T>,
}

impl<I, T, E> Parser<I, T>
where
    I: Iterator<Item = Result<T, E>>,
{
    pub(crate) fn new(iter: I) -> Self {
        Parser {
            iter,
            next_item: None,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        assert!(self.next_item.is_none());
        self.next_item = Some(item);
    }

    pub(crate) fn pop(&mut self) -> Result<Option<T>, E> {
        if let Some(next) = self.next_item.take() {
            return Ok(Some(next));
        }
        if let Some(item) = self.iter.next() {
            item.map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn peek(&mut self) -> Result<Option<&T>, E> {
        if let Some(ref next) = self.next_item {
            return Ok(Some(next));
        }
        self.next_item = if let Some(item) = self.iter.next() {
            Some(item?)
        } else {
            None
        };
        Ok(self.next_item.as_ref())
    }
}
