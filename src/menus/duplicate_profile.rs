// Duplicate Submenu

use iced::{widget, Element, Task};

use crate::application::{Application, Message};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DuplicateData {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum DuplicateSubmenuMessage {
    EditName(String),
    Done,
}

impl Into<Message> for DuplicateSubmenuMessage {
    fn into(self) -> Message {
        Message::DuplicateProfileMessage(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DuplicateSubmenuData {
    pub original: String,
    pub name: String,
}

impl DuplicateSubmenuData {
    pub fn new(original: &str) -> Self {
        let mut name = original.to_string();
        name.push_str(" (Copy)");
        Self {
            original: original.to_string(),
            name,
        }
    }

    pub fn update(&mut self, m: DuplicateSubmenuMessage) -> Task<Message> {
        match m {
            DuplicateSubmenuMessage::EditName(s) => {
                self.name = s;
                Task::none()
            }
            DuplicateSubmenuMessage::Done => {
                let name = self.name.trim().to_string();
                let data = DuplicateData { name };
                Task::done(Message::DuplicateCurrentProfile(data))
                    .chain(Task::done(Message::ButtonExitSubmenuPressed))
            }
        }
    }

    pub fn view<'a>(&'a self, root: &'a Application) -> Element<'a, DuplicateSubmenuMessage> {
        let name_test = self.name.trim();
        let done_button_msg = (!name_test.eq(&self.original)).then_some(DuplicateSubmenuMessage::Done);
        widget::column![
            widget::text("Duplicate"),
            widget::row![
                widget::text_input("--Name--", &self.name).on_input(DuplicateSubmenuMessage::EditName),
            ]
            .spacing(5),
            widget::vertical_space(),
            widget::button("Done").on_press_maybe(done_button_msg)
        ]
        .spacing(5)
        .padding(5)
        .into()
    }
}
