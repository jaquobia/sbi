use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub mod popups;
pub mod component;
pub mod widgets;

/// Stolen from ratatui Demo2 example and improved
/// simple helper method to split an area into multiple pre-defined sub-areas
pub fn layout<const N: usize>(area: Rect, direction: Direction, constraints: [Constraint; N]) -> [Rect; N] {
    let areas = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area).to_vec();
    <[Rect; N]>::try_from(areas).unwrap()
}

pub fn center_box(area: Rect, width: Constraint, height: Constraint) -> Rect {
    let [_padding_top, box_vertical, _padding_bottom] = layout(area, Direction::Vertical, [Constraint::Fill(1), height, Constraint::Fill(1)]);
    let [_padding_left, box_area, _padding_right] = layout(box_vertical, Direction::Horizontal, [Constraint::Fill(1), width, Constraint::Fill(1)]);
    box_area
}
