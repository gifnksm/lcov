use super::{ParseRecordError, Record};
use std::io::{self, BufRead, Lines};

#[derive(Debug)]
pub struct Reader<B> {
    lines: Lines<B>,
    line: u32,
}

impl<B> Reader<B>
where
    B: BufRead,
{
    pub fn new(buf: B) -> Reader<B> {
        Reader {
            lines: buf.lines(),
            line: 1,
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)] Io(#[cause] io::Error),
    #[fail(display = "invalid record syntax at line {}: {}", _0, _1)]
    ParseRecord(u32, #[cause] ParseRecordError),
}

impl<B> Iterator for Reader<B>
where
    B: BufRead,
{
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| {
            line.map_err(Error::Io).and_then(|line| {
                self.line += 1;
                line.parse().map_err(|e| Error::ParseRecord(self.line, e))
            })
        })
    }
}
