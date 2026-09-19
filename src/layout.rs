use ratatui::layout::{Constraint, Layout, Position, Rect};

/// Below this width only one pane is shown at a time.
pub const NARROW_WIDTH: u16 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pane {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaneRects {
    pub left: Option<Rect>,
    pub right: Option<Rect>,
    pub key_bar: Rect,
}

/// Pane rectangles for `area`. On a narrow terminal the focused pane is the one shown.
pub fn compute(area: Rect, has_right: bool, focus: Pane) -> PaneRects {
    let [body, key_bar] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    if !has_right {
        return PaneRects {
            left: Some(body),
            right: None,
            key_bar,
        };
    }
    if area.width < NARROW_WIDTH {
        return match focus {
            Pane::Left => PaneRects {
                left: Some(body),
                right: None,
                key_bar,
            },
            Pane::Right => PaneRects {
                left: None,
                right: Some(body),
                key_bar,
            },
        };
    }
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(body);
    PaneRects {
        left: Some(left),
        right: Some(right),
        key_bar,
    }
}

/// The pane containing a terminal cell, if any.
pub fn pane_at(rects: &PaneRects, column: u16, row: u16) -> Option<Pane> {
    let hit = |r: Option<Rect>| r.is_some_and(|r| r.contains(Position::new(column, row)));
    if hit(rects.left) {
        Some(Pane::Left)
    } else if hit(rects.right) {
        Some(Pane::Right)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_terminal_splits_sixty_forty_above_key_bar() {
        let r = compute(Rect::new(0, 0, 120, 30), true, Pane::Left);
        assert_eq!(r.left.unwrap().width, 72);
        assert_eq!(r.right.unwrap().width, 48);
        assert_eq!(r.left.unwrap().height, 29);
        assert_eq!(r.key_bar, Rect::new(0, 29, 120, 1));
    }

    #[test]
    fn no_right_pane_gives_left_the_full_body() {
        let r = compute(Rect::new(0, 0, 120, 30), false, Pane::Right);
        assert_eq!(r.left, Some(Rect::new(0, 0, 120, 29)));
        assert_eq!(r.right, None);
    }

    #[test]
    fn narrow_terminal_shows_only_the_focused_pane() {
        let area = Rect::new(0, 0, 80, 30);
        let left = compute(area, true, Pane::Left);
        assert_eq!((left.left.map(|r| r.width), left.right), (Some(80), None));
        let right = compute(area, true, Pane::Right);
        assert_eq!((right.left, right.right.map(|r| r.width)), (None, Some(80)));
    }

    #[test]
    fn hit_testing_finds_the_pane_under_a_cell() {
        let r = compute(Rect::new(0, 0, 120, 30), true, Pane::Left);
        assert_eq!(pane_at(&r, 10, 5), Some(Pane::Left));
        assert_eq!(pane_at(&r, 100, 5), Some(Pane::Right));
        assert_eq!(pane_at(&r, 10, 29), None);
    }
}
