use crossterm::event::{self, Event, KeyCode};
use ratatui::{buffer::Buffer, layout::{Direction, Rect}, style::{Color, Style}, widgets::{Block, Borders, Clear, Paragraph, Widget}};
use tui_textarea::TextArea;

use crate::{app::AppMessage, json::InstanceDataJson, tui::ui::{self, component::UIComponent, widgets::spinner::Spinner}};

use super::ConsumablePopup;

pub struct NewInstancePopup {
    instance_name: TextArea<'static>,
    exec_spinner: ui::widgets::spinner::Spinner,
    collection_text: TextArea<'static>,
    option_interactable: u16,
}

impl NewInstancePopup {
    /// Create a NewInstance popup with default values
    pub fn new<S: Into<String>>(executables: Vec<S>) -> Self {
        let mut text_name = TextArea::default();
        text_name.set_cursor_line_style(Style::default());
        let mut items = vec![String::from("Default")];
        items.extend(executables.into_iter().map(|s|s.into()));
        let mut exec_spinner = Spinner::new(items);
        exec_spinner.set_arrow_style(Style::default().fg(Color::Red));
        exec_spinner.set_arrow_unavilable_style(Style::default().fg(Color::Reset));
        exec_spinner.set_option_style(Style::default());
        exec_spinner.set_hide_unavailable_move(true);

        let mut this = Self {
            instance_name: text_name,
            exec_spinner,
            collection_text: TextArea::default(),
            option_interactable: 0,
        };
        Self::update(&mut this);
        this
    }

    /// Collect text-area text into string
    pub fn get_instance_name(&self) -> String {
        self.instance_name.lines().join("")
    }

    /// Collect text-area text into string
    pub fn get_collection_id(&self) -> String {
        self.collection_text.lines().join("")
    }

    /// Convert spinner index into executable
    pub fn get_executable(&self) -> Option<String> {
        match self.exec_spinner.get_option_index() {
            0 => { None },
            _ => { Some(self.exec_spinner.get_option_value().to_string()) },
        }
    }

    /// Reapply the styles for stateful widgets (text area and spinner)
    fn update(&mut self) {
        let border_style = 
            if !super::is_instance_name_valid(&self.get_instance_name()) {
                Style::default().fg(Color::Red)
            } else if self.option_interactable == 0 {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default()
            };
        let border_title_style = if self.option_interactable == 0 {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default()
            };
        let block = Block::default()
            .title("Instance Name")
            .title_style(border_title_style)
            .borders(Borders::ALL)
            .border_style(border_style);
        self.instance_name.set_block(block);
        let spinner_style = Style::default().fg(if self.option_interactable == 1 { Color::LightYellow } else { Color::default() });
        self.exec_spinner.set_option_style(spinner_style);

        let collection_border_style = 
            if self.option_interactable == 2 {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default()
            };
        let collection_border_title_style = if self.option_interactable == 2 {
            Style::default().fg(Color::LightYellow)
        } else {
            Style::default()
        };
        let collection_block = Block::default()
            .title("Collection ID")
            .title_style(collection_border_title_style)
            .borders(Borders::all())
            .border_style(collection_border_style);
        self.collection_text.set_block(collection_block);

    }

}

impl UIComponent<AppMessage> for NewInstancePopup {
    fn ui(&self, buffer: &mut Buffer, area: Rect) {
        use ratatui::layout::Constraint as C;

        // Styles
        let new_instance_text_style = Style::default();
        // let new_instance_block_style = Style::default().bg(Color::DarkGray);
        let new_instance_block_style = Style::default().bg(Color::Indexed(233));
        let ok_button_style = Style::default().fg(if self.option_interactable == 3 { Color::LightYellow } else { Color::default() });

        // Shadow area so the only drawable region is the centered rect
        let area = ui::center_box(area, C::Percentage(32), C::Percentage(32));
        Clear.render(area, buffer);
        let border = Block::default()
            .borders(Borders::ALL)
            .style(new_instance_block_style)
            .border_type(ratatui::widgets::BorderType::Rounded);

        let area_inner_border = border.inner(area);
        let [ area_input_name_text, area_spinner_executable_title, area_spinner_executable, area_collection_id, area_ok_button, _padding] = 
            ui::layout(area_inner_border, Direction::Vertical, [C::Length(3), C::Length(1), C::Length(1), C::Length(3), C::Length(1), C::Min(0)]);

        // Draw shit
        border.render(area, buffer);
        self.instance_name.widget().render(area_input_name_text, buffer);
        Paragraph::new("Executable:").centered().style(new_instance_text_style).render(area_spinner_executable_title, buffer);
        self.exec_spinner.widget().render(area_spinner_executable, buffer);
        self.collection_text.widget().render(area_collection_id, buffer);
        Paragraph::new("[Ok]").style(ok_button_style).right_aligned().render(area_ok_button, buffer);
    }

    fn handle_event(&mut self, event: &Event) -> Option<AppMessage> {
        match event {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                // TODO: handle paste events with custom keybinds
                match key.code {
                    // Press enter over [Ok] button
                    KeyCode::Enter if self.option_interactable == 3 => {
                        if super::is_instance_name_valid(&self.get_instance_name()) {
                            return Some(AppMessage::ClosePopup);
                        }
                    },
                    KeyCode::Enter => { }, // noop
                    KeyCode::Up => {
                        self.option_interactable = self.option_interactable.saturating_sub(1);
                        self.update();
                    },
                    KeyCode::Down => {
                        self.option_interactable = self.option_interactable.saturating_add(1).min(3);
                        self.update();
                    },
                    _ => {
                        match self.option_interactable {
                            // Name Text area
                            0 => { if self.instance_name.input_without_shortcuts(*key) { self.update(); } },
                            // Spinner
                            1 => { self.exec_spinner.input(*key); },
                            // Collection ID Text area
                            2 => { if self.collection_text.input(*key) { self.update(); } }
                            _ => {},
                        }
                    }
                }
            },
            _ => {}
        }
        None
    }
}

impl ConsumablePopup<AppMessage> for NewInstancePopup {
    fn consume(&mut self) -> Option<AppMessage> {
        let name = self.get_instance_name();
        let executable = self.get_executable();
        let collection_id = self.get_collection_id();
        let collection_id = (!collection_id.is_empty()).then_some(collection_id);
        let instance_data_json = InstanceDataJson { name, executable, additional_assets: None, collection_id };
        Some(AppMessage::CreateInstance(instance_data_json))
    }
}
