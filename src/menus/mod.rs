use std::path::PathBuf;

use iced::{widget, Element, Task};

use crate::{
    application::{Application, Message},
    profile::{Profile, ProfileJson},
};

// New Profile Submenu

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NewProfileSubmenuMessage {
    TextFieldEditName(String),
    TextFieldEditCollectionID(String),
    ToggleLinkMods(bool),
    CreateProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewProfileSubmenu {
    pub name: String,
    pub collection_id: String,
    pub link_mods: bool,
}

impl NewProfileSubmenu {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            collection_id: String::from(""),
            link_mods: false,
        }
    }

    pub fn update(&mut self, message: NewProfileSubmenuMessage) -> Task<Message> {
        match message {
            NewProfileSubmenuMessage::TextFieldEditName(s) => {
                self.name = s;
                Task::none()
            }
            NewProfileSubmenuMessage::TextFieldEditCollectionID(s) => {
                self.collection_id = s;
                Task::none()
            }
            NewProfileSubmenuMessage::ToggleLinkMods(b) => {
                self.link_mods = b;
                Task::none()
            }
            NewProfileSubmenuMessage::CreateProfile => {
                let profile = {
                    let collection_id =
                        (!self.collection_id.is_empty()).then(|| self.collection_id.clone());
                    // Make a new profile with just a name
                    crate::profile::ProfileJson {
                        name: self.name.clone(),
                        additional_assets: None,
                        collection_id,
                        link_mods: false,
                    }
                };
                Task::done(Message::CreateProfile(profile))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let create_button_message = (!self.name.is_empty()).then_some(Message::NewProfileMessage(
            NewProfileSubmenuMessage::CreateProfile,
        ));
        widget::column![
            widget::column![
                widget::text("New Profile"),
                widget::text_input("-Name-", &self.name).on_input(|s| {
                    Message::NewProfileMessage(NewProfileSubmenuMessage::TextFieldEditName(s))
                }),
                widget::text_input("-Collection ID-", &self.collection_id).on_input(|s| {
                    Message::NewProfileMessage(NewProfileSubmenuMessage::TextFieldEditCollectionID(
                        s,
                    ))
                }),
                widget::checkbox("Link vanilla mods folder", self.link_mods).on_toggle(|b| {
                    Message::NewProfileMessage(NewProfileSubmenuMessage::ToggleLinkMods(b))
                })
            ]
            .spacing(8),
            widget::vertical_space(),
            widget::row![
                widget::button("Back").on_press(Message::ButtonExitSubmenuPressed),
                widget::horizontal_space(),
                widget::button("Create").on_press_maybe(create_button_message),
            ]
        ]
        .padding(5)
        .into()
    }
}

// Settings Submenu

#[derive(Debug, Clone)]
pub enum SettingsSubmenuMessage {
    ClickExecutableListItem(String),
    ClickAddExecutableButton,
    ClickRemoveExecutableButton,
    NewExecutableNameInput(String),
    ClickNewExecutableButton,
    NewExecutableSelected(Option<PathBuf>),
    ClickNewExecutableAssetsButton,
    NewExecutableAssetsSelected(Option<PathBuf>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingsSubmenuData {
    pub selected_executable: Option<String>,
    new_executable_name: String,
    new_executable_path: Option<PathBuf>,
    new_executable_assets: Option<PathBuf>,
}

impl SettingsSubmenuData {
    pub fn new() -> Self {
        Self {
            selected_executable: None,
            new_executable_name: String::new(),
            new_executable_path: None,
            new_executable_assets: None,
        }
    }

    pub fn update(&mut self, m: SettingsSubmenuMessage) -> Task<Message> {
        match m {
            SettingsSubmenuMessage::ClickExecutableListItem(s) => {
                self.selected_executable = Some(s);
                Task::none()
            }
            SettingsSubmenuMessage::ClickAddExecutableButton => {
                let name = self.new_executable_name.clone();
                let path = self.new_executable_path.clone().unwrap();
                let assets = self
                    .new_executable_assets
                    .as_ref()
                    .map(|p| p.parent().unwrap().to_owned());
                Task::done(Message::CreateExecutable(name, path, assets))
            }
            SettingsSubmenuMessage::ClickRemoveExecutableButton => {
                let name = self
                    .selected_executable
                    .take()
                    .expect("REMOVE_BUTTON_CLICKED_WITHOUT_SELECTED_EXECUTABLE");
                Task::done(Message::RemoveExecutable(name))
            }
            SettingsSubmenuMessage::NewExecutableNameInput(s) => {
                self.new_executable_name = s;
                Task::none()
            }
            SettingsSubmenuMessage::ClickNewExecutableButton => {
                async fn pick_executable() -> Option<PathBuf> {
                    let file: Option<rfd::FileHandle> =
                        rfd::AsyncFileDialog::new().pick_file().await;
                    file.map(|f| f.path().to_path_buf())
                }
                Task::perform(pick_executable(), |r| {
                    Message::SettingsMessage(SettingsSubmenuMessage::NewExecutableSelected(r))
                })
            }
            SettingsSubmenuMessage::NewExecutableSelected(mby_file) => {
                if let Some(path) = mby_file {
                    log::info!("Picked file {}", path.display());
                    self.new_executable_path = Some(path);
                }
                Task::none()
            }
            SettingsSubmenuMessage::ClickNewExecutableAssetsButton => {
                async fn pick_executable_assets() -> Option<PathBuf> {
                    let file: Option<rfd::FileHandle> = rfd::AsyncFileDialog::new()
                        .add_filter("Pak", &["pak"])
                        .pick_file()
                        .await;
                    file.map(|f| f.path().to_path_buf())
                }
                Task::perform(pick_executable_assets(), |r| {
                    Message::SettingsMessage(SettingsSubmenuMessage::NewExecutableAssetsSelected(r))
                })
            }
            SettingsSubmenuMessage::NewExecutableAssetsSelected(mby_file) => {
                if let Some(path) = mby_file {
                    log::info!("Picked file {}", path.display());
                    self.new_executable_assets = Some(path);
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self, root: &'a Application) -> Element<'a, Message> {
        let executable_to_element =
            |(i, executable_name): (usize, &'a String)| -> (usize, Element<'a, Message>) {
                let color = self
                    .selected_executable
                    .as_ref()
                    .filter(|selected| selected.as_str() == executable_name.as_str())
                    .and(Some(iced::color!(0x00ff00)));
                let clickable = widget::mouse_area(widget::row![
                    widget::text(executable_name).color_maybe(color)
                ])
                .on_press(Message::SettingsMessage(
                    SettingsSubmenuMessage::ClickExecutableListItem(executable_name.to_string()),
                ));
                (i, clickable.into())
            };
        let executables = widget::keyed_column(
            root.executables()
                .keys()
                .enumerate()
                .map(executable_to_element),
        );

        let add_button_action =
            (self.new_executable_path.is_some() && !self.new_executable_name.is_empty()).then_some(
                Message::SettingsMessage(SettingsSubmenuMessage::ClickAddExecutableButton),
            );
        let remove_button_action = self
            .selected_executable
            .as_ref()
            .map(|_| Message::SettingsMessage(SettingsSubmenuMessage::ClickRemoveExecutableButton));

        widget::column![
            widget::text("Settings"),
            widget::column![
                widget::text("Executables"),
                widget::horizontal_rule(2),
                widget::scrollable(executables),
                widget::horizontal_rule(2)
            ],
            widget::row![
                widget::button("Add").on_press_maybe(add_button_action),
                widget::button("Modify"),
                widget::button("Remove").on_press_maybe(remove_button_action)
            ]
            .spacing(5),
            widget::text_input("-Name-", &self.new_executable_name).on_input(|s| {
                Message::SettingsMessage(SettingsSubmenuMessage::NewExecutableNameInput(s))
            }),
            widget::row![
                widget::button("Pick Executable").on_press(Message::SettingsMessage(
                    SettingsSubmenuMessage::ClickNewExecutableButton
                )),
                widget::text!("{:?}", self.new_executable_path),
            ]
            .spacing(5),
            widget::row![
                widget::button("Pick Assets").on_press(Message::SettingsMessage(
                    SettingsSubmenuMessage::ClickNewExecutableAssetsButton
                )),
                widget::text!("{:?}", self.new_executable_assets),
            ]
            .spacing(5),
            widget::vertical_space(),
            widget::button("Close").on_press(Message::ButtonExitSubmenuPressed)
        ]
        .spacing(5)
        .padding(5)
        .into()
    }
}

// Configure Profile Submenu

#[derive(Debug, Clone)]
pub enum ConfigureProfileSubmenuMessage {
    ToggleLinkModsCheckbox(bool),
    SaveAndExit,
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
            M::ToggleLinkModsCheckbox(b) => {
                self.profile_copy.link_mods = b;
                Task::none()
            }
            M::SaveAndExit => Task::done(Message::ModifyCurrentProfile(self.profile_copy.clone())),
        }
    }

    pub fn view<'a>(&'a self, root: &'a Application) -> Element<'a, Message> {
        type M = ConfigureProfileSubmenuMessage;
        widget::column![
            widget::column![widget::text("Configuring Profile"),].spacing(8),
            widget::checkbox("Link mods", self.profile_copy.link_mods)
                .on_toggle(|b| Message::ConfigureProfileMessage(M::ToggleLinkModsCheckbox(b))),
            widget::vertical_space(),
            widget::row![
                widget::button("Close").on_press(Message::ButtonExitSubmenuPressed),
                widget::horizontal_space(),
                widget::button("Save").on_press(Message::ConfigureProfileMessage(M::SaveAndExit)),
            ]
        ]
        .padding(5)
        .into()
    }
}
