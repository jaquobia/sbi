// Rename Submenu

use iced::{widget, Element, Task};

use crate::application::{Application, Message};

#[derive(Debug, Clone)]
pub enum RenameSubmenuMessage {
    EditName(String),
    Done,
}

impl Into<Message> for RenameSubmenuMessage {
    fn into(self) -> Message {
        Message::RenameProfileMessage(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenameSubmenuData {
    pub name: String,
}

impl RenameSubmenuData {
    pub fn new(original: &str) -> Self {
        Self {
            name: original.to_string(),
        }
    }

    pub fn update(&mut self, m: RenameSubmenuMessage) -> Task<Message> {
        match m {
            RenameSubmenuMessage::EditName(s) => {
                self.name = s;
                Task::none()
            }
            RenameSubmenuMessage::Done => {
                Task::done(Message::RenameCurrentProfile(self.name.clone()))
                    .chain(Task::done(Message::ButtonExitSubmenuPressed))
            }
        }
    }

    pub fn view<'a>(&'a self, root: &'a Application) -> Element<'a, RenameSubmenuMessage> {
        widget::column![
            widget::text("Rename"),
            widget::row![
                widget::text_input("--Name--", &self.name).on_input(RenameSubmenuMessage::EditName),
            ]
            .spacing(5),
            widget::vertical_space(),
            widget::button("Done").on_press(RenameSubmenuMessage::Done)
        ]
        .spacing(5)
        .padding(5)
        .into()
    }
}
