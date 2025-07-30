// Configure Profile Submenu

use iced::{widget, Element, Task};

use crate::{
    application::{Application, Message},
    profile::ProfileJson,
};

#[derive(Debug, Clone)]
pub enum ConfigureProfileSubmenuMessage {
    Exit,
    ToggleLinkModsCheckbox(bool),
    Delete,
}

impl Into<Message> for ConfigureProfileSubmenuMessage {
    fn into(self) -> Message {
        Message::ConfigureProfileMessage(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConfigureProfileSubmenuData {
    profile_copy: ProfileJson,
}

impl ConfigureProfileSubmenuData {
    pub fn new(original: &ProfileJson) -> Self {
        Self {
            profile_copy: original.clone(),
        }
    }

    pub fn update(&mut self, m: ConfigureProfileSubmenuMessage) -> Task<Message> {
        type M = ConfigureProfileSubmenuMessage;
        match m {
            M::Exit => Task::done(Message::ButtonExitSubmenuPressed),
            M::ToggleLinkModsCheckbox(b) => {
                self.profile_copy.link_mods = b;
                Task::done(Message::ModifyCurrentProfile(self.profile_copy.clone()))
            }
            M::Delete => Task::done(Message::DeleteCurrentProfile)
                .chain(Task::done(Message::ButtonExitSubmenuPressed)),
        }
    }

    pub fn view<'a>(
        &'a self,
        _root: &'a Application,
    ) -> Element<'a, ConfigureProfileSubmenuMessage> {
        type M = ConfigureProfileSubmenuMessage;
        widget::column![
            widget::column![widget::text("Configuring Profile"),].spacing(8),
            widget::checkbox("Link mods", self.profile_copy.link_mods)
                .on_toggle(|b| M::ToggleLinkModsCheckbox(b)),
            widget::vertical_space(),
            widget::row![
                widget::button("Close").on_press(M::Exit),
                widget::horizontal_space(),
                widget::button("Delete").on_press(M::Delete),
            ]
        ]
        .padding(5)
        .into()
    }
}
