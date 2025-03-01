use core::panic;
use std::str::Chars;

pub(crate) struct PutBackChars<'a> {
    iter: Chars<'a>,
    buffer: Option<char>,
}

impl<'a> From<Chars<'a>> for PutBackChars<'a> {
    fn from(value: Chars<'a>) -> Self {
        Self {
            iter: value,
            buffer: None,
        }
    }
}

impl<'a> PutBackChars<'a> {
    pub fn next(&mut self) -> Option<char> {
        if let Some(out) = self.buffer {
            self.buffer = None;
            Some(out)
        } else {
            self.iter.next()
        }
    }
    pub fn putback(&mut self, value: char) {
        if let Some(out) = self.buffer {
            panic!("Cannot put {value} back into the buffer. It already contains {out}")
        }
        self.buffer = Some(value);
    }
}
