/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
//! The crate-wide [`Error`] type. All fallible operations of this crate
//! return it instead of panicking.

use std::fmt::{self, Display, Formatter};

/// Errors that can occur while visualizing audio data.
#[derive(Debug)]
pub enum Error {
    /// The input can't be visualized, e.g. it is empty or contains
    /// non-finite values.
    InvalidData(String),
    /// Writing the output file failed.
    Io(std::io::Error),
    /// Rendering or encoding the chart failed.
    Chart(charts_rs::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "invalid input data: {msg}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Chart(e) => write!(f, "chart rendering error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidData(_) => None,
            Self::Io(e) => Some(e),
            Self::Chart(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<charts_rs::Error> for Error {
    fn from(e: charts_rs::Error) -> Self {
        Self::Chart(e)
    }
}
