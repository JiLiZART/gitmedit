/// One display row of a wrapped line.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Char index in the source line where this row starts.
    pub start: usize,
    /// Text shown on this row. A space the line was broken at is not included.
    pub text: String,
}

/// Wrap `line` into rows of at most `width` chars, breaking at the last space that fits and
/// hard-breaking words wider than `width`. Always returns at least one segment.
// ponytail: counts chars, not terminal cells; wide CJK/emoji chars can overflow a row.
// Switch to unicode-width (already pulled in by ratatui) if that gets reported.
pub fn wrap(line: &str, width: usize) -> Vec<Segment> {
    let chars: Vec<char> = line.chars().collect();
    if width == 0 || chars.len() <= width {
        return vec![Segment {
            start: 0,
            text: line.to_string(),
        }];
    }
    let mut out = Vec::new();
    let mut start = 0;
    while chars.len() - start > width {
        let end = start + width;
        match (start + 1..=end).rev().find(|&i| chars[i] == ' ') {
            Some(space) => {
                out.push(Segment {
                    start,
                    text: chars[start..space].iter().collect(),
                });
                start = space + 1;
            }
            None => {
                out.push(Segment {
                    start,
                    text: chars[start..end].iter().collect(),
                });
                start = end;
            }
        }
    }
    out.push(Segment {
        start,
        text: chars[start..].iter().collect(),
    });
    out
}

/// Map a char column of the source line to (row, column within that row).
pub fn locate(segments: &[Segment], col: usize) -> (usize, usize) {
    let row = segments.iter().rposition(|s| s.start <= col).unwrap_or(0);
    (row, col - segments[row].start)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(segments: &[Segment]) -> Vec<&str> {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    #[test]
    fn short_line_is_one_segment() {
        assert_eq!(
            wrap("short", 40),
            vec![Segment {
                start: 0,
                text: "short".into()
            }]
        );
    }

    #[test]
    fn empty_line_is_one_empty_segment() {
        assert_eq!(texts(&wrap("", 40)), vec![""]);
    }

    #[test]
    fn zero_width_does_not_wrap() {
        assert_eq!(texts(&wrap("any text", 0)), vec!["any text"]);
    }

    #[test]
    fn breaks_at_last_space_that_fits() {
        let s = wrap("hello world foo bar baz", 11);
        assert_eq!(texts(&s), vec!["hello world", "foo bar baz"]);
        assert_eq!(s[1].start, 12);
    }

    #[test]
    fn hard_breaks_words_wider_than_width() {
        assert_eq!(
            texts(&wrap("abcdefghijklmnop", 5)),
            vec!["abcde", "fghij", "klmno", "p"]
        );
    }

    #[test]
    fn rows_never_exceed_width_on_multibyte_text() {
        for seg in wrap("cafe\u{0301} is good and long enough", 6) {
            assert!(seg.text.chars().count() <= 6, "{seg:?}");
        }
    }

    #[test]
    fn no_text_is_lost() {
        let line = "the quick brown fox jumps over the lazy dog";
        let rows: Vec<String> = wrap(line, 10).into_iter().map(|s| s.text).collect();
        assert_eq!(rows.join(" "), line);
    }

    #[test]
    fn locate_maps_columns_to_rows() {
        let s = wrap("hello world foo", 10);
        assert_eq!(texts(&s), vec!["hello", "world foo"]);
        assert_eq!(locate(&s, 0), (0, 0));
        assert_eq!(locate(&s, 5), (0, 5));
        assert_eq!(locate(&s, 6), (1, 0));
        assert_eq!(locate(&s, 15), (1, 9));
    }
}
