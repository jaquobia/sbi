use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{layout::Direction, style::{Color, Style}, text::Text, widgets::{Block, Borders, Clear, Paragraph, Widget}};

use crate::{app::AppMessage, ui::{self, component::UIComponent}};

use super::ConsumablePopup;

pub struct ConfirmationPopup<T> {
    message: Option<Box<T>>,
    text: String,
    button_index: u8,
}

impl<T> ConfirmationPopup<T> {
    pub fn new(message: T, text: String) -> Self {
        Self {
            message: Some(Box::new(message)),text,button_index: 0
        }
    }
}

impl UIComponent<AppMessage> for ConfirmationPopup<AppMessage> {
    fn handle_event(&mut self, event: &crossterm::event::Event) -> Option<AppMessage> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return None; }
            match key.code {
                KeyCode::Enter if self.button_index == 0 => {
                    return Some(AppMessage::ClosePopupNoOp);
                },
                KeyCode::Enter if self.button_index == 1 => {
                    return Some(AppMessage::ClosePopup);
                },
                KeyCode::Left if self.button_index == 0 => {
                    self.button_index = 1;
                },
                KeyCode::Right if self.button_index == 1 => {
                    self.button_index = 0;
                },
                _ => {}
            }
        }
        None
    }
    fn ui(&self, buffer: &mut ratatui::prelude::Buffer, area: ratatui::prelude::Rect) {
        use ratatui::layout::Constraint as C;
        let confirmation_border_style = Style::default().bg(Color::Indexed(233));
        let confirmation_text_style = Style::default();
        let confirmation_button_unselected_style = Style::default();
        let confirmation_button_selected_style = Style::default().fg(Color::LightYellow);
        let (ok_button_style, cancel_button_style) = if self.button_index == 0 {
            (confirmation_button_unselected_style, confirmation_button_selected_style)
        } else {
            (confirmation_button_selected_style, confirmation_button_unselected_style)
        };
        let text = Text::from(self.text.as_str()).style(confirmation_text_style);
        let text_width: u16 = text.width() as u16; 
        let text_height: u16 = text.height() as u16; 
        let text_paragraph = Paragraph::new(text).centered();
        let button_cancel_text = "[Cancel]";
        let button_ok_text = "[Ok]";
        let button_text_width = (button_cancel_text.len() + button_ok_text.len()) as u16;
        let width = C::Length(u16::max(button_text_width, text_width) + 4);
        let height = C::Length(4 + text_height);
        let dialog_area = ui::center_box(area, width, height);
        let border = Block::default().borders(Borders::ALL).title("Confirmation").style(confirmation_border_style).border_type(ratatui::widgets::BorderType::Rounded);
        Clear.render(dialog_area, buffer);
        let dialog_area_inner = border.inner(dialog_area);
        border.render(dialog_area, buffer);
        let [text_area, _paddng_area, buttons_area] = ui::layout(dialog_area_inner, Direction::Vertical, [C::Length(1); 3]);
        text_paragraph.render(text_area, buffer);
        let [_padding_left, button_ok_area, _padding_center, button_cancel_area, _padding_right] = 
            ui::layout(buttons_area, Direction::Horizontal, 
                // TODO: Figure out what to do about casting to u16
                [C::Min(0), C::Length(button_ok_text.len() as u16), C::Fill(1), C::Length(button_cancel_text.len() as u16), C::Min(0)]
            );
        Paragraph::new(button_ok_text).style(ok_button_style).render(button_ok_area, buffer);
        Paragraph::new(button_cancel_text).style(cancel_button_style).render(button_cancel_area, buffer);
    }
}

impl ConsumablePopup<AppMessage> for ConfirmationPopup<AppMessage> {
    fn consume(&mut self) -> Option<AppMessage> {
        self.message.take().map(|v|*v)
    }
}
