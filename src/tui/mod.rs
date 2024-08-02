use std::io::Stdout;

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use itertools::Either;
use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Direction, Margin, Rect}, prelude::CrosstermBackend, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{
        Block, BorderType, Borders, List, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    }, Frame, Terminal
};
use throbber_widgets_tui::ThrobberState;

use crate::app::AppSBI;

pub mod ui;

// Funcitons taken from Ratatui guide
fn restore_tui() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn init_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
        println!();
    }));
}

pub fn setup() -> std::io::Result<()> {
    init_panic_hook();

    crossterm::terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

pub fn tear_down() -> std::io::Result<()> {
    restore_tui()
}

/// Rendering structure
pub struct RenderStorage {
    pub working_throbber: ThrobberState,
}

impl RenderStorage {
    pub fn new() -> Self {
        Self {
            working_throbber: ThrobberState::default(),
        }
    }

    pub fn draw(&mut self, app: &AppSBI, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
        terminal.draw(|frame| Self::ui(frame, &app, self))?;
        Ok(())
    }

    pub fn fixed_update(&mut self) {
        self.working_throbber.calc_next();
    }

    fn draw_keys(&mut self, area: Rect, buffer: &mut Buffer, keys: &[(&str, &str)]) {
        let keybind_key_style = Style::new();
        let keybind_desc_style = Style::new();
        let keybind_separator_style = Style::new().fg(Color::Yellow);

        let separator = Span::styled("|", keybind_separator_style);
        let keys_last = keys.len() - 1;
        let key_spans = keys
            .iter()
            .enumerate()
            .flat_map(|(i, (key, desc))| {
                let key = Span::styled(format!(" {}:", key), keybind_key_style);
                let desc = Span::styled(format!("{} ", desc), keybind_desc_style);
                if i < keys_last {
                    Either::Left([key, desc, separator.clone()].into_iter())
                } else {
                    Either::Right([key, desc].into_iter())
                }
            })
            .collect::<Vec<_>>();
        Paragraph::new(Line::from(key_spans))
            .alignment(Alignment::Center)
            .fg(Color::Indexed(236))
            .bg(Color::Indexed(232))
            .render(area, buffer);
    }

    fn draw_home(
        &mut self,
        area: Rect,
        buffer: &mut Buffer,
        app: &AppSBI,
    ) {
        use Constraint as C;
        let title_style = Style::new();
        let highlighted_instance_style = Style::new()
            .bg(if !app.is_task_running() {
                Color::White
            } else {
                Color::Indexed(240)
            })
            .fg(Color::Black);
        let home_border_style = Style::new().fg(Color::Green);
        let instance_list_style = Style::new().fg(Color::White);
        let instance_info_style = Style::new().fg(Color::White);

        // Draw borders and Title
        Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .title("SBI")
            .title_alignment(Alignment::Left)
            .title_style(title_style)
            .style(home_border_style)
            .bg(Color::Indexed(233))
            .render(area, buffer);
        {
            let current_instance = if let Ok(instance) = app.get_instance_current() {
                instance
            } else {
                return;
            };
            let executable = match current_instance.executable() {
                Some(executable) => executable.to_string(),
                None => {
                    format!("Default({})", app.config.default_executable)
                }
            };
            let line_1 = Line::from(vec![
                Span::styled("Name: ", instance_info_style),
                Span::styled(current_instance.name(), instance_info_style),
            ]);
            let line_2 = Line::from(vec![
                Span::styled("Executable: ", instance_info_style),
                Span::styled(executable, instance_info_style),
            ]);
            let lines = vec![line_1, line_2];
            let lines_count = lines.len();
            let text = Text::from(lines);

            let [area_instance_list, area_line_separator, area_instance_info, area_line_separator2, area_bg_task_indicator, _] =
                ui::layout(
                    area.inner(Margin {
                        vertical: 1,
                        horizontal: 1,
                    }),
                    Direction::Vertical,
                    [
                        C::Min(0),
                        C::Length(1),
                        C::Length(lines_count as u16),
                        C::Length(1),
                        C::Length(1),
                        C::Length(0),
                    ],
                );
            if !app.instances.is_empty() {
                let items = app.get_instances().iter().map(|ins| ins.name());
                let list = List::new(items)
                    .style(instance_list_style)
                    .highlight_style(highlighted_instance_style);
                let mut state = ListState::default().with_selected(Some(app.instance_index));
                StatefulWidget::render(list, area_instance_list, buffer, &mut state);
            }
            Block::new()
                .borders(Borders::TOP)
                .style(home_border_style)
                .render(area_line_separator, buffer);
            Block::new()
                .borders(Borders::TOP)
                .style(home_border_style)
                .render(area_line_separator2, buffer);
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .render(area_instance_info, buffer);
            if app.is_task_running() {
                let throbber = throbber_widgets_tui::Throbber::default()
                    .label("Working.. Input will be disabled until work is done.")
                    .throbber_set(throbber_widgets_tui::ASCII);
                StatefulWidget::render(
                    throbber,
                    area_bg_task_indicator,
                    buffer,
                    &mut self.working_throbber,
                );
            }
        }
    }

    fn ui(frame: &mut Frame, app: &AppSBI, render_storage: &mut RenderStorage) {
        let area = frame.size();
        let buffer = frame.buffer_mut();

        use Constraint as C;

        let [area_instances, area_keybinds] =
            ui::layout(area, Direction::Vertical, [C::Min(0), C::Length(1)]);
        render_storage.draw_home(area_instances, buffer, app);

        // Draw Status and Keybinds
        let keys = [
            ("Q", "Quit"),
            ("↑/k", "Up"),
            ("↓/j", "Down"),
            ("Enter", "Run Options"),
            ("n", "New Instance"),
            ("m", "Modify Instance"),
        ];
        render_storage.draw_keys(area_keybinds, buffer, &keys);
        if let Some(popup) = &app.popup {
            popup.borrow().ui(buffer, area);
        }
    }
}
