// New Profile Submenu

use iced::{widget, Element, Task};

use crate::application::Message;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NewProfileSubmenuMessage {
    Exit,
    TextFieldEditName(String),
    TextFieldEditCollectionID(String),
    ToggleLinkMods(bool),
    CreateProfile,
}

impl Into<Message> for NewProfileSubmenuMessage {
    fn into(self) -> Message {
        Message::NewProfileMessage(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewProfileSubmenuData {
    pub name: String,
    pub collection_id: String,
    pub link_mods: bool,
}

impl NewProfileSubmenuData {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            collection_id: String::from(""),
            link_mods: false,
        }
    }

    pub fn update(&mut self, message: NewProfileSubmenuMessage) -> Task<Message> {
        type M = NewProfileSubmenuMessage;
        match message {
            M::Exit => Task::done(Message::ButtonExitSubmenuPressed),
            M::TextFieldEditName(s) => {
                self.name = s;
                Task::none()
            }
            M::TextFieldEditCollectionID(s) => {
                self.collection_id = s;
                Task::none()
            }
            M::ToggleLinkMods(b) => {
                self.link_mods = b;
                Task::none()
            }
            M::CreateProfile => {
                let profile = {
                    let collection_id =
                        (!self.collection_id.is_empty()).then(|| self.collection_id.clone());
                    // Make a new profile with just a name
                    crate::profile::ProfileJson {
                        name: self.name.clone(),
                        additional_assets: None,
                        collection_id,
                        link_mods: false,
                        selected_executable: None,
                    }
                };
                Task::done(Message::CreateProfile(profile))
            }
        }
    }

    pub fn view(&self) -> Element<'_, NewProfileSubmenuMessage> {
        type M = NewProfileSubmenuMessage;
        let create_button_message = (!self.name.is_empty()).then_some(M::CreateProfile);
        widget::column![
            widget::column![
                widget::text("New Profile"),
                widget::text_input("-Name-", &self.name).on_input(|s| { M::TextFieldEditName(s) }),
                widget::text_input("-Collection ID-", &self.collection_id)
                    .on_input(|s| { M::TextFieldEditCollectionID(s) }),
                widget::checkbox("Link vanilla mods folder", self.link_mods)
                    .on_toggle(|b| { M::ToggleLinkMods(b) })
            ]
            .spacing(8),
            widget::vertical_space(),
            widget::row![
                widget::button("Back").on_press(M::Exit),
                widget::horizontal_space(),
                widget::button("Create").on_press_maybe(create_button_message),
            ]
        ]
        .padding(5)
        .into()
    }
}
