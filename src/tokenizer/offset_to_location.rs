//! Byte offset to line/column mapping for source positions.
//!
//! Lazily computes line and column arrays on first access, then
//! provides O(1) lookups from byte offset to `(line, column)`.

use super::char_code_definitions::is_bom;

/// A source location with offset, line, and column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Source filename (if provided).
    pub source: Option<String>,
    /// Byte offset into the source.
    pub offset: usize,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

/// A source range with start and end locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationRange {
    /// Source filename.
    pub source: Option<String>,
    /// Start location.
    pub start: LocationPoint,
    /// End location.
    pub end: LocationPoint,
}

/// A point in a source range (offset + line + column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationPoint {
    /// Byte offset into the source.
    pub offset: usize,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

/// Maps byte offsets to line/column positions in a CSS source string.
///
/// Line and column arrays are computed lazily on first lookup.
#[derive(Debug)]
pub struct OffsetToLocation {
    source: String,
    start_offset: usize,
    start_line: u32,
    start_column: u32,
    lines: Vec<u32>,
    columns: Vec<u32>,
    computed: bool,
}

impl OffsetToLocation {
    /// Create a new mapper for the given source string.
    pub fn new(source: &str, start_offset: usize, start_line: u32, start_column: u32) -> Self {
        Self {
            source: source.to_string(),
            start_offset,
            start_line,
            start_column,
            lines: Vec::new(),
            columns: Vec::new(),
            computed: false,
        }
    }

    /// Get the location for a byte offset.
    pub fn get_location(&mut self, offset: usize, filename: Option<&str>) -> Location {
        self.ensure_computed();
        Location {
            source: filename.map(String::from),
            offset: self.start_offset + offset,
            line: self.lines.get(offset).copied().unwrap_or(self.start_line),
            column: self.columns.get(offset).copied().unwrap_or(self.start_column),
        }
    }

    /// Get a location range for a start..end byte range.
    pub fn get_location_range(
        &mut self,
        start: usize,
        end: usize,
        filename: Option<&str>,
    ) -> LocationRange {
        self.ensure_computed();
        LocationRange {
            source: filename.map(String::from),
            start: LocationPoint {
                offset: self.start_offset + start,
                line: self.lines.get(start).copied().unwrap_or(self.start_line),
                column: self.columns.get(start).copied().unwrap_or(self.start_column),
            },
            end: LocationPoint {
                offset: self.start_offset + end,
                line: self.lines.get(end).copied().unwrap_or(self.start_line),
                column: self.columns.get(end).copied().unwrap_or(self.start_column),
            },
        }
    }

    fn ensure_computed(&mut self) {
        if !self.computed {
            self.compute_lines_and_columns();
        }
    }

    fn compute_lines_and_columns(&mut self) {
        let bytes = self.source.as_bytes();
        let source_length = bytes.len();

        let start_offset = self.source.chars().next()
            .map_or(0, |ch| is_bom(u32::from(ch)) * ch.len_utf8());

        self.lines.resize(source_length + 1, 0);
        self.columns.resize(source_length + 1, 0);

        let mut line = self.start_line;
        let mut column = self.start_column;

        let mut i = start_offset;
        while i < source_length {
            let code = bytes[i];

            self.lines[i] = line;
            self.columns[i] = column;
            column += 1;

            // Newline handling: LF (0x0A), CR (0x0D), FF (0x0C)
            if code == 0x0A || code == 0x0D || code == 0x0C {
                // CR+LF counts as one newline
                if code == 0x0D && i + 1 < source_length && bytes[i + 1] == 0x0A {
                    i += 1;
                    self.lines[i] = line;
                    self.columns[i] = column;
                }
                line += 1;
                column = 1;
            }

            i += 1;
        }

        // Sentinel for EOF position
        self.lines[source_length] = line;
        self.columns[source_length] = column;

        self.computed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line() {
        let mut loc = OffsetToLocation::new("abc", 0, 1, 1);
        let l = loc.get_location(0, None);
        assert_eq!(l.line, 1);
        assert_eq!(l.column, 1);

        let l = loc.get_location(2, None);
        assert_eq!(l.line, 1);
        assert_eq!(l.column, 3);
    }

    #[test]
    fn multi_line_lf() {
        let mut loc = OffsetToLocation::new("a\nb\nc", 0, 1, 1);
        assert_eq!(loc.get_location(0, None).line, 1); // a
        assert_eq!(loc.get_location(2, None).line, 2); // b
        assert_eq!(loc.get_location(4, None).line, 3); // c
    }

    #[test]
    fn crlf_newline() {
        let mut loc = OffsetToLocation::new("a\r\nb", 0, 1, 1);
        assert_eq!(loc.get_location(0, None).line, 1); // a
        assert_eq!(loc.get_location(3, None).line, 2); // b
    }

    #[test]
    fn custom_start() {
        let mut loc = OffsetToLocation::new("abc", 10, 5, 3);
        let l = loc.get_location(0, None);
        assert_eq!(l.offset, 10);
        assert_eq!(l.line, 5);
        assert_eq!(l.column, 3);
    }

    #[test]
    fn location_range() {
        let mut loc = OffsetToLocation::new("a\nb", 0, 1, 1);
        let r = loc.get_location_range(0, 2, Some("test.css"));
        assert_eq!(r.source, Some("test.css".to_string()));
        assert_eq!(r.start.line, 1);
        assert_eq!(r.start.column, 1);
        assert_eq!(r.end.line, 2);
        assert_eq!(r.end.column, 1);
    }

    #[test]
    fn eof_position() {
        let mut loc = OffsetToLocation::new("ab", 0, 1, 1);
        let l = loc.get_location(2, None);
        assert_eq!(l.line, 1);
        assert_eq!(l.column, 3);
    }
}
