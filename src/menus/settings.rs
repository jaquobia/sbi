// Settings Submenu

use std::path::PathBuf;

use iced::{widget, Element, Task};

use crate::application::{Application, Message};

#[derive(Debug, Clone)]
pub enum SettingsSubmenuMessage {
    Exit,
    ClickExecutableListItem(String),
    ClickAddExecutableButton,
    ClickRemoveExecutableButton,
    NewExecutableNameInput(String),
    ClickNewExecutableButton,
    NewExecutableSelected(Option<PathBuf>),
    ClickNewExecutableAssetsButton,
    NewExecutableAssetsSelected(Option<PathBuf>),
    ToggleCloseOnLaunch(bool),
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
        type M = SettingsSubmenuMessage;
        match m {
            M::Exit => Task::done(Message::ButtonExitSubmenuPressed),
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
            SettingsSubmenuMessage::ToggleCloseOnLaunch(state) => {
                Task::done(Message::ToggleCloseOnLaunch(state))
            }
        }
    }

    pub fn view<'a>(&'a self, root: &'a Application) -> Element<'a, SettingsSubmenuMessage> {
        type M = SettingsSubmenuMessage;
        let executable_to_element =
            |(i, executable_name): (usize, &'a String)| -> (usize, Element<'a, SettingsSubmenuMessage>) {
                let color = self
                    .selected_executable
                    .as_ref()
                    .filter(|selected| selected.as_str() == executable_name.as_str())
                    .and(Some(iced::color!(0x00ff00)));
                let clickable = widget::mouse_area(widget::row![
                    widget::text(executable_name).color_maybe(color)
                ])
                .on_press(M::ClickExecutableListItem(
                    executable_name.to_string(),
                ));
                (i, clickable.into())
            };
        let executables = widget::keyed_column(
            root.executables()
                .keys()
                .enumerate()
                .map(executable_to_element),
        );

        let add_button_action = (self.new_executable_path.is_some()
            && !self.new_executable_name.is_empty())
        .then_some(M::ClickAddExecutableButton);
        let remove_button_action = self
            .selected_executable
            .as_ref()
            .map(|_| M::ClickRemoveExecutableButton);

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
            widget::text_input("-Name-", &self.new_executable_name)
                .on_input(|s| { M::NewExecutableNameInput(s) }),
            widget::row![
                widget::button("Pick Executable").on_press(M::ClickNewExecutableButton),
                widget::text!("{:?}", self.new_executable_path),
            ]
            .spacing(5),
            widget::row![
                widget::button("Pick Assets").on_press(M::ClickNewExecutableAssetsButton),
                widget::text!("{:?}", self.new_executable_assets),
            ]
            .spacing(5),
            widget::checkbox("Close on Launch", root.config().close_on_launch)
                .on_toggle(M::ToggleCloseOnLaunch),
            widget::vertical_space(),
            widget::button("Close").on_press(M::Exit)
        ]
        .spacing(5)
        .padding(5)
        .into()
    }
}
