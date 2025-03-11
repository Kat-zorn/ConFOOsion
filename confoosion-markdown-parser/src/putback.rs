use std::str::Chars;

pub struct PutBackChars<'a> {
    internal: UnmarkedPutBackChars<'a>,
    pub line_number: usize,
    pub column_number: usize,
}

struct UnmarkedPutBackChars<'a> {
    iter: Chars<'a>,
    buffer: Vec<char>,
}

impl<'a> From<Chars<'a>> for UnmarkedPutBackChars<'a> {
    fn from(value: Chars<'a>) -> Self {
        Self {
            iter: value,
            buffer: Vec::new(),
        }
    }
}

impl<'a> PutBackChars<'a> {
    pub fn next(&mut self) -> Option<char> {
        let ch = self.internal.next();
        if ch == Some('\n') {
            self.line_number += 1;
            self.column_number = 1;
        } else {
            self.column_number += 1;
        }
        ch
    }

    pub fn putback(&mut self, value: char) {
        self.internal.putback(value);
        if value == '\n' {
            self.line_number -= 1;
        } else {
            self.column_number -= 1;
        }
    }

    pub fn putback_maybe(&mut self, value: Option<char>) {
        if let Some(value) = value {
            self.putback(value);
        }
    }
}

impl<'a> UnmarkedPutBackChars<'a> {
    pub fn next(&mut self) -> Option<char> {
        if let Some(out) = self.buffer.pop() {
            Some(out)
        } else {
            self.iter.next()
        }
    }
    pub fn putback(&mut self, value: char) {
        self.buffer.push(value);
    }
}

impl<'a> From<UnmarkedPutBackChars<'a>> for PutBackChars<'a> {
    fn from(value: UnmarkedPutBackChars<'a>) -> Self {
        Self {
            internal: value,
            line_number: 1,
            column_number: 1,
        }
    }
}

impl<'a> From<Chars<'a>> for PutBackChars<'a> {
    fn from(value: Chars<'a>) -> Self {
        Self {
            internal: value.into(),
            line_number: 1,
            column_number: 1,
        }
    }
}
