use std::fs;
use std::path::PathBuf;

use crate::common::Span;

#[derive(Debug, Clone)]
pub enum SourceFrom {
    Path(PathBuf),
    String,
}

impl SourceFrom {
    pub fn display(&self) -> String {
        match self {
            Self::Path(path) => format!("{}", path.display()),
            Self::String => "string".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub from: SourceFrom,
    pub src: Vec<u8>,
    pub line_starts: Vec<usize>,
}

impl Source {
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();

        let src = fs::read_to_string(&path).map_err(|why| {
            format!("Failed to open source {}: {}", path.display(), why)
        })?;

        let mut src = src.into_bytes();
        let line_starts = Self::compute_line_starts(&src);

        src.push(0);

        Ok(Self {
            from: SourceFrom::Path(path),
            src,
            line_starts,
        })
    }

    pub fn from_string(src: impl Into<String>) -> Result<Self, String> {
        let mut src = src.into().into_bytes();
        let line_starts = Self::compute_line_starts(&src);

        src.push(0);

        Ok(Self {
            from: SourceFrom::String,
            src,
            line_starts,
        })
    }

    pub fn get(&self, idx: usize) -> u8 {
        self.src.get(idx).copied().unwrap_or(0u8)
    }

    pub fn eof(&self) -> Span {
        let eof = self.src.len() - 1;
        Span::new(eof, eof)
    }

    fn compute_line_starts(src: &Vec<u8>) -> Vec<usize> {
        let mut starts: Vec<usize> = vec![0];

        for (idx, byte) in src.iter().enumerate() {
            if idx == src.len() - 1 {
                continue;
            }

            if *byte == b'\n' {
                starts.push(idx + 1);
            }
        }

        starts
    }

    pub fn line_from_index(&self, idx: usize) -> Option<usize> {
        if idx == self.src.len() {
            return Some(self.line_starts.len() - 1);
        }

        let line = self.line_starts.partition_point(|&start| start <= idx);

        line.checked_sub(1)
    }

    pub fn line(&self, line: usize) -> &str {
        let start = self.line_starts[line - 1];

        let mut end = if line == self.line_starts.len() {
            self.src.len()
        } else {
            self.line_starts[line]
        };

        if end > start && self.src[end - 1] == b'\n' {
            end -= 1;
        }

        std::str::from_utf8(&self.src[start..end])
            .expect("UTF-8 error when attempting to read source")
    }

    pub fn line_col_from_index(&self, index: usize) -> (usize, usize) {
        let line = self
            .line_from_index(index)
            .or_else(|| panic!("line index out of bounds!"))
            .unwrap();

        let line_start = self.line_starts[line];

        let col = index - line_start;

        (line + 1, col + 1)
    }

    pub fn index_from_line_col(&self, line: usize, col: usize) -> usize {
        self.line_starts[line] + col
    }

    pub fn view(&self, start: usize, mut end: usize) -> &str {
        if end < start {
            end = start;
        }
        std::str::from_utf8(&self.src[start..end]).unwrap()
    }

    pub fn view_span<T: AsRef<Span>>(&self, span: T) -> &str {
        let span = span.as_ref();
        self.view(span.start, span.end)
    }
}
