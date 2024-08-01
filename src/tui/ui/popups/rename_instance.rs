use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{style::Style, widgets::{Block, BorderType, Borders, Widget}};
use tui_textarea::TextArea;

use crate::{app::AppMessage, instance::ModifyInstance, tui::ui::{self, component::UIComponent} };

use super::ConsumablePopup;

pub struct RenamePopup {
    name: TextArea<'static>
}

impl RenamePopup {
    pub fn new() -> Self {
        let mut name = TextArea::default();
        name.set_block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Rename"));
        name.set_cursor_line_style(Style::default());
        Self {
            name
        }
    }
}

impl UIComponent<AppMessage> for RenamePopup {
    fn handle_event(&mut self, event: &crossterm::event::Event) -> Option<AppMessage> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Enter if key.kind == KeyEventKind::Press => {
                    return Some(AppMessage::ClosePopup);
                }
                _ => {}
            }
            self.name.input_without_shortcuts(*key);
        }
        None
    }
    fn ui(&self, buffer: &mut ratatui::prelude::Buffer, area: ratatui::prelude::Rect) {
        use ratatui::layout::Constraint as C;
        let area = ui::center_box(area, C::Length(22), C::Length(3));
        self.name.widget().render(area, buffer);
    }
}

impl ConsumablePopup<AppMessage> for RenamePopup {
    fn consume(&mut self) -> Option<AppMessage> {
        let name = self.name.lines().join("");
        Some(AppMessage::ModifyInstance(vec![ModifyInstance::Name(name)]))
    }
}
