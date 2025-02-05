use ratatui::crossterm::event::Event;
use ratatui::{buffer::Buffer, layout::Rect};

pub trait UIComponent<T> {
    fn ui(&self, buffer: &mut Buffer, area: Rect);
    fn handle_event(&mut self, event: &Event) -> Option<T>;
}
