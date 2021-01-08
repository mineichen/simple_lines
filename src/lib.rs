
#![deny(missing_docs)]

//! # Simple and secure line iterators
//! Simple line iterator which prevents OutOfMemory if an attacker inputs very long sequences without a delimiter by applying a max_capacity. 
//! The implementation reuses the last `std::rc::Rc<String>` on calling next() if it isnt used anymore.
//! 
//! It currently uses the linebuffer library under the hood to provid a much simpler interface with fewer pitfalls:
//!  - Implements `std::iter::Iterator`
//!  - Incomplete lines result in `Err(Incomplete<Rc<String>>)` to force users to think about this scenario
//!  - Ok variant should be compatible with `std::io::BufReader` (beside wrapping in Rc)
//!  - Invalid UTF8 results in `Err(Encoding)`
use {
    std::{io::Read, rc::Rc},
    linereader::LineReader
};

mod bound;

/// Extensions to std::io::Read to implement simple and secure line iterators
pub trait ReadExt {
    /// Underlying Reader
    type Read: std::io::Read;
    /// Creates a RcLineIterator with a custom buffer capacity
    fn lines_rc_with_capacity(self, buffer_capacity: usize) -> bound::RcLineIterator<Self::Read>;
    /// Creates a RcLineIterator with the default capacity of 64kb
    /// ```
    /// use reflines::ReadExt;
    ///
    /// let cursor = std::io::Cursor::new("12345678\r\n123");
    /// let mut lines = cursor.lines_rc_with_capacity(5);
    /// if let reflines::Error::Incomplete(x) = lines.next().unwrap().unwrap_err() {
    ///     assert_eq!(*x, "12345");
    /// } else {
    ///     panic!("Expected incomplete if EOL was not detected");
    /// }
    /// if let reflines::Error::Incomplete(x) = lines.next().unwrap().unwrap_err() {
    ///     assert_eq!(*x, "678");
    /// } else {
    ///     panic!("Expected incomplete line for the rest");
    /// }
    /// assert_eq!(*lines.next().unwrap().unwrap(), "123");
    /// ```
    fn lines_rc(self) -> bound::RcLineIterator<Self::Read>;
}

impl<T: Read> ReadExt for T {
    type Read = T;
    fn lines_rc(self) -> bound::RcLineIterator<T> {
        self.lines_rc_with_capacity(64*1024)
    }
    fn lines_rc_with_capacity(self, buffer_capacity: usize) -> bound::RcLineIterator<Self::Read> {
        bound::RcLineIterator::new(
            LineReader::with_capacity(buffer_capacity, self),
            buffer_capacity
        )
    }
}

/// Result of calling ReadExt::lines_rc
#[derive(thiserror::Error, Debug)]
pub enum Error<T: AsRef<String> + std::fmt::Debug> {
    /// Forwarded Errors from the underlying reader
    #[error("io")]
    Io(#[from] std::io::Error),
    /// If a line contains any invalid UTF8 character
    #[error("encoding")]
    Encoding(#[from] std::str::Utf8Error),
    /// If the provided buffer is full, it's content is returned as `Incomplete`.
    /// The rest of the line, including the last part containing the linebreak, will all be `Incomplete` or other errors.
    #[error("Incomplete line")]
    Incomplete(T),
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufRead, Cursor};
    use super::*;

    #[test]
    fn return_whitespace_line() {
        assert_behave_same(&" \r\n");
        assert_behave_same(&" \n");
    }
    #[test]
    fn return_whitespace_on_last_line() {
        assert_behave_same(&"\r\n ");
        assert_behave_same(&"\n ");
    }
    #[test]
    fn return_trailing_double_linebkreak() {
        assert_behave_same(&"\r\n\r\n");
        assert_behave_same(&"\n\n");
    }
    #[test]
    fn return_whitespace_line_and_terminating_a() {
        assert_behave_same(&"\r\na")
    }

    #[test]
    fn assert_non_ascii_returns_error() {
        let buf = ['a' as u8, 'b' as u8, 254];
        assert_behave_same(&buf);
    }

    fn assert_behave_same<T: AsRef<[u8]>>(input: &T) {
        let mut own_iter = BufReader::new(Cursor::new(input)).lines();
        let mut rc_iter = std::io::Cursor::new(input).lines_rc();
        for (own_line, rc_line) in own_iter.by_ref().zip(rc_iter.by_ref()) {
            match own_line {
                Ok(o) => assert_eq!(o, *rc_line.unwrap()),
                Err(_) => { rc_line.unwrap_err(); }
            }
        }
        assert_eq!(own_iter.count(), 0);
        assert_eq!(rc_iter.count(), 0);
    }
}
