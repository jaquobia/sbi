use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{layout::{Direction, Margin}, style::{Color, Style}, text::Text, widgets::{Block, BorderType, Borders, Paragraph, Widget}};
use tui_textarea::TextArea;

use crate::{app::AppMessage, instance::{Instance, ModifyInstance}, tui::ui::{self, component::UIComponent, widgets::spinner::Spinner}};

use super::ConsumablePopup;

pub struct ModifyInstancePopup {
    instance: Instance,
    exec_spinner: ui::widgets::spinner::Spinner,
    collection_text: TextArea<'static>,
    option_interactable: usize,
}

impl ModifyInstancePopup {
    pub fn new<S: Into<String>>(instance: Instance, executables: Vec<S>) -> Self {
        let mut items = Vec::with_capacity(executables.len() + 1);
        items.push(String::from("Default"));
        items.extend(executables.into_iter().map(S::into));
        let instance_executable = instance.executable().clone().unwrap_or_else(||String::from("Default"));
        let index = items.iter().position(|t|instance_executable.eq(t)).unwrap_or(0);
        let mut exec_spinner = Spinner::new(items);
        exec_spinner.set_arrow_style(Style::default().fg(Color::Red));
        exec_spinner.set_arrow_unavilable_style(Style::default().fg(Color::Reset));
        exec_spinner.set_option_style(Style::default());
        exec_spinner.set_hide_unavailable_move(true);

        exec_spinner.set_option_index(index);

        let collection_text = TextArea::new(vec![instance.collection_id().unwrap_or_default()]);
        
        let mut this = Self {
            instance,
            exec_spinner,
            collection_text,
            option_interactable: 0,
        };
        Self::update(&mut this);
        this
    }
    /// Collect text-area text into string
    pub fn get_collection_id(&self) -> String {
        self.collection_text.lines().join("")
    }

    pub fn update(&mut self) {

        let spinner_style = Style::default().fg(if self.option_interactable == 0 { Color::LightYellow } else { Color::default() });
        self.exec_spinner.set_option_style(spinner_style);

        let collection_border_style = 
            if !self.is_valid() {
                Style::default().fg(Color::Red)
            } else if self.option_interactable == 1 {
                Style::default().fg(Color::LightYellow)
            } else {
                Style::default()
            };
        let collection_border_title_style = if self.option_interactable == 1 {
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

    pub fn is_valid(&mut self) -> bool {
        true
    }
}

impl UIComponent<AppMessage> for ModifyInstancePopup {
    fn handle_event(&mut self, event: &crossterm::event::Event) -> Option<AppMessage> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { return None; }
            match key.code {
                KeyCode::Enter if self.option_interactable == 2 => {
                    if self.is_valid() {
                        return Some(AppMessage::ClosePopup);
                    }
                },
                KeyCode::Enter => { }, // noop
                KeyCode::Up => {
                    self.option_interactable = self.option_interactable.saturating_sub(1);
                    self.update();
                },
                KeyCode::Down => {
                    self.option_interactable = self.option_interactable.saturating_add(1).min(2);
                    self.update();
                }
                _ => { 
                    match self.option_interactable {
                        0 => { self.exec_spinner.input(*key); },
                        1 => { if self.collection_text.input(*key) { self.update(); } },
                        _ => {},
                    }
                }
            }
        }
        None
    }
    fn ui(&self, buffer: &mut ratatui::prelude::Buffer, area: ratatui::prelude::Rect) {
        use ratatui::layout::Constraint as C;
        let area = ui::center_box(area, C::Percentage(32), C::Percentage(32));
        let border = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Configure Instance");
        border.render(area, buffer);
        let area = area.inner(Margin { horizontal:1, vertical:1 });
        let [_, spinner_area, _, collection_area, confirmation_button_area] = ui::layout(area, Direction::Vertical, [C::Max(1), C::Length(1), C::Max(1), C::Length(3), C::Length(1)]);
        {
            let exec_label_string = " Exec:";
            let exec_label = Text::from(exec_label_string);
            let exec_label_length: u16 = exec_label_string.len().try_into().unwrap_or_default();
            let [spinner_label_area, spinner_area] = ui::layout(spinner_area, Direction::Horizontal, 
                [C::Length(exec_label_length), C::Fill(1)]);
            exec_label.render(spinner_label_area, buffer);
            self.exec_spinner.widget().render(spinner_area, buffer);
        }

        self.collection_text.widget().render(collection_area, buffer);
        
        {
            let ok_button_style = Style::default().fg(if self.option_interactable == 2 { Color::LightYellow } else { Color::default() });
            Paragraph::new("[Ok]").style(ok_button_style).right_aligned().render(confirmation_button_area, buffer);
        }
    }
}

impl ConsumablePopup<AppMessage> for ModifyInstancePopup {
    fn consume(&mut self) -> Option<AppMessage> {
        let executable = (self.exec_spinner.get_option_index() > 0).then(||self.exec_spinner.get_option_value().to_string());
        let mut modifications = vec![];
        modifications.push(ModifyInstance::Executable(executable));
        let collection_id = self.get_collection_id();
        let collection_id = (!collection_id.is_empty()).then_some(collection_id);
        modifications.push(ModifyInstance::Collection(collection_id));
        Some(AppMessage::ModifyInstance(modifications))
    }
}
