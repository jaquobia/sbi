use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Spinner {
    options: Vec<String>,
    selected: usize,
    style_move_available: Style,
    style_move_unavailable: Style,
    style_option_text: Style,
    spinner_arrow_left: String,
    spinner_arrow_right: String,
    hide_unavailable_move: bool,
}

impl Spinner {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: 0,
            style_move_available: Style::default(),
            style_move_unavailable: Style::default().fg(Color::Reset),
            style_option_text: Style::default(),
            spinner_arrow_left: "<".to_string(),
            spinner_arrow_right: ">".to_string(),
            hide_unavailable_move: false,
        }
    }
    pub fn spin_left(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn spin_right(&mut self) {
        self.selected = self.selected.saturating_add(1).min(self.options.len() - 1);
    }
    pub fn set_option_style<S: Into<Style>>(&mut self, style: S) {
        self.style_option_text = style.into();
    }
    pub fn set_arrow_style<S: Into<Style>>(&mut self, style: S) {
        self.style_move_available = style.into();
    }
    pub fn set_arrow_unavilable_style<S: Into<Style>>(&mut self, style: S) {
        self.style_move_unavailable = style.into();
    }
    pub fn set_hide_unavailable_move(&mut self, flag: bool) {
        self.hide_unavailable_move = flag;
    }
    pub fn get_option_value(&self) -> &str {
        &self.options[self.selected]
    }
    pub fn get_option_index(&self) -> usize {
        self.selected
    }
    pub fn set_option_index(&mut self, index: usize) {
        self.selected = index;
    }
    /// Generic Left/Right input handler
    /// Don't use if you don't want Left/Right arrow keys, Use spin_left() and spin_right()
    /// instead
    /// Returns true if an event was processed
    pub fn input(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Left => {
                self.spin_left();
                true
            }
            KeyCode::Right => {
                self.spin_right();
                true
            }
            _ => false,
        }
    }
}

impl Widget for &Spinner {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (left_arrow_style, left_arrow) = if self.selected == 0 {
            (
                &self.style_move_unavailable,
                if self.hide_unavailable_move {
                    " "
                } else {
                    self.spinner_arrow_left.as_str()
                },
            )
        } else {
            (&self.style_move_available, self.spinner_arrow_left.as_str())
        };
        let (right_arrow_style, right_arrow) = if self.selected == self.options.len() - 1 {
            (
                &self.style_move_unavailable,
                if self.hide_unavailable_move {
                    " "
                } else {
                    self.spinner_arrow_right.as_str()
                },
            )
        } else {
            (
                &self.style_move_available,
                self.spinner_arrow_right.as_str(),
            )
        };

        let spinner_text = Line::from(vec![
            Span::styled(left_arrow, *left_arrow_style),
            Span::raw(" "),
            Span::styled(self.get_option_value(), self.style_option_text),
            Span::raw(" "),
            Span::styled(right_arrow, *right_arrow_style),
        ]);
        Paragraph::new(spinner_text).centered().render(area, buf);
    }
}
