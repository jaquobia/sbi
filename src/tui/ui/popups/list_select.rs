use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{style::{Color, Style}, text::{Line, Text}, widgets::{Block, Borders, Clear, Paragraph, Widget}};

use crate::{app::AppMessage, tui::ui::{self, component::UIComponent}};

use super::ConsumablePopup;

pub struct ListSelectPopup<T> {
    selected: usize,
    items: Vec<(String, T)>,
    text_style: Style,
    highlighted_text_style: Style,
    block: Block<'static>
}

#[allow(unused)]
impl ListSelectPopup<AppMessage> {
    pub fn new<S: Into<String>>(items: Vec<(S, AppMessage)>) -> Self {
        let block = Block::default().borders(Borders::ALL);
        let items = items.into_iter().map(|(string, message)| (string.into(), message)).collect();
        Self {
            items,
            selected:0,
            text_style: Style::default(),
            highlighted_text_style: Style::default().fg(Color::LightYellow),
            block
        }
    }
    pub fn set_text_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.text_style = style.into();
        self
    }
    pub fn set_highlighted_text_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlighted_text_style = style.into();
        self
    }
    pub fn set_block(mut self, block: Block<'static>) -> Self {
        self.block = block;
        self
    }

}

impl UIComponent<AppMessage> for ListSelectPopup<AppMessage> {
    fn handle_event(&mut self, event: &crossterm::event::Event) -> Option<AppMessage> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return None; }
            match key.code {
                KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                },
                KeyCode::Down => {
                    self.selected = self.selected.saturating_add(1).min(self.items.len()-1);
                },
                KeyCode::Enter if self.selected < self.items.len() => {
                    return Some(AppMessage::ClosePopup);
                }
                _ => {}
            }
        }
        None
    }
    fn ui(&self, buffer: &mut ratatui::prelude::Buffer, area: ratatui::prelude::Rect) {
        use ratatui::layout::Constraint as C;
        let width = C::Percentage(40);
        let height = C::Length(2 + self.items.len() as u16);
        let dialog_area = ui::center_box(area, width, height);
        let block = &self.block;

        let area = block.inner(dialog_area);
        Clear.render(dialog_area, buffer);
        block.render(dialog_area, buffer);

        let lines = self.items.iter()
            .enumerate()
            .map(|(index, (text, _))| Line::styled(text, if self.selected == index { self.highlighted_text_style } else { self.text_style }))
            .collect::<Vec<_>>();
        
        Paragraph::new(Text::from(lines)).render(area, buffer);
    }
}

impl ConsumablePopup<AppMessage> for ListSelectPopup<AppMessage> {
    fn consume(&mut self) -> Option<AppMessage> {
        if self.items.len() > self.selected {
            let (_item, message) = self.items.remove(self.selected);
            return Some(message);
        }
        None
    }
}
