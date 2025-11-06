// Settings Submenu

use std::path::PathBuf;

use iced::{widget, Element, Task};

use crate::{
    application::{Application, Message},
    config::SBIConfig,
    executable::{Executable, ExecutableVariant},
    SBIDirectories,
};

#[derive(Debug, Clone)]
pub enum SettingsSubmenuMessage {
    Exit,
    SelectExecutable(String),
    GenerateExectuable,
    DeleteExecutable,
    EditExecutableName(String),

    PickExecutableBinary,
    PickedExecutableBinary(Option<PathBuf>),
    PickExecutableAssets,
    PickedExecutableAssets(Option<PathBuf>),
    SelectExecutableVariant(ExecutableVariant),
    ToggleCloseOnLaunch(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingsSubmenuData {
    pub selected_executable: Option<String>,
    new_executable_name: String,
    // new_executable_path: Option<PathBuf>,
    // new_executable_assets: Option<PathBuf>,
    // new_executable_variant: Option<ExecutableVariant>,
}

impl SettingsSubmenuData {
    pub fn new() -> Self {
        Self {
            selected_executable: None,
            new_executable_name: String::new(),
            // new_executable_path: None,
            // new_executable_assets: None,
            // new_executable_variant: Some(ExecutableVariant::default()),
        }
    }

    pub fn update(
        &mut self,
        m: SettingsSubmenuMessage,
        config: &mut SBIConfig,
        dirs: &SBIDirectories,
    ) -> Task<Message> {
        type M = SettingsSubmenuMessage;
        match m {
            M::Exit => Task::done(Message::ButtonExitSubmenuPressed),
            SettingsSubmenuMessage::SelectExecutable(s) => {
                self.selected_executable = Some(s);
                Task::none()
            }
            SettingsSubmenuMessage::GenerateExectuable => {
                let name = self.new_executable_name.clone();
                // let path = self.new_executable_path.clone().unwrap();
                let path = dirs.data_directory.clone();
                // let assets = self
                //     .new_executable_assets
                //     .as_ref()
                //     .map(|p| p.parent().unwrap().to_owned());
                let assets = None;
                let variant = ExecutableVariant::Vanilla;
                let executable = Executable {
                    bin: path,
                    assets,
                    variant,
                };
                Task::done(Message::WriteExecutable(name, executable))
            }
            SettingsSubmenuMessage::DeleteExecutable => {
                let name = self
                    .selected_executable
                    .take()
                    .expect("REMOVE_BUTTON_CLICKED_WITHOUT_SELECTED_EXECUTABLE");
                Task::done(Message::RemoveExecutable(name))
            }
            SettingsSubmenuMessage::EditExecutableName(s) => {
                self.new_executable_name = s;
                Task::none()
            }
            SettingsSubmenuMessage::PickExecutableBinary => {
                async fn pick_executable() -> Option<PathBuf> {
                    let file: Option<rfd::FileHandle> =
                        rfd::AsyncFileDialog::new().pick_file().await;
                    file.map(|f| f.path().to_path_buf())
                }
                Task::perform(pick_executable(), |r| {
                    Message::SettingsMessage(SettingsSubmenuMessage::PickedExecutableBinary(r))
                })
            }
            SettingsSubmenuMessage::PickedExecutableBinary(mby_file) => {
                if let Some(path) = mby_file {
                    log::info!("Picked file {}", path.display());
                    if let Some(name) = self.selected_executable.as_ref() {
                        if let Some(executable) = config.get_executable_mut(name) {
                            executable.bin = path;
                            return Task::done(Message::WriteExecutable(
                                name.clone(),
                                executable.clone(),
                            ));
                        };
                    }
                    // self.new_executable_path = Some(path);
                }
                Task::none()
            }
            SettingsSubmenuMessage::PickExecutableAssets => {
                async fn pick_executable_assets() -> Option<PathBuf> {
                    let file: Option<rfd::FileHandle> = rfd::AsyncFileDialog::new()
                        .add_filter("Pak", &["pak"])
                        .pick_file()
                        .await;
                    file.map(|f| f.path().to_path_buf())
                }
                Task::perform(pick_executable_assets(), |r| {
                    Message::SettingsMessage(SettingsSubmenuMessage::PickedExecutableAssets(r))
                })
            }
            SettingsSubmenuMessage::PickedExecutableAssets(mby_file) => {
                log::info!("Picked file {mby_file:?}");
                if let Some(name) = self.selected_executable.as_ref() {
                    if let Some(executable) = config.get_executable_mut(name) {
                        executable.assets = mby_file;
                        return Task::done(Message::WriteExecutable(
                            name.clone(),
                            executable.clone(),
                        ));
                    };
                }
                // self.new_executable_assets = Some(path);
                Task::none()
            }
            SettingsSubmenuMessage::SelectExecutableVariant(variant) => {
                // self.new_executable_variant.replace(variant);
                if let Some(name) = self.selected_executable.as_ref() {
                    if let Some(executable) = config.get_executable_mut(name) {
                        executable.variant = variant;
                        return Task::done(Message::WriteExecutable(
                            name.clone(),
                            executable.clone(),
                        ));
                    };
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
                .on_press(M::SelectExecutable(
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

        let selected_executable = self
            .selected_executable
            .as_ref()
            .and_then(|n| root.config().get_executable(n));
        let (
            remove_button_action,
            executable_variant,
            executable_bin,
            executable_assets,
            pick_binary_action,
            pick_assets_action,
        ) = if let Some(executable) = selected_executable {
            (
                Some(M::DeleteExecutable),
                executable.variant.clone(),
                Some(executable.bin.clone()),
                executable.assets.clone(),
                Some(M::PickExecutableBinary),
                Some(M::PickExecutableAssets),
            )
        } else {
            (None, ExecutableVariant::Vanilla, None, None, None, None)
        };
        let pick_variant = widget::pick_list(
            ExecutableVariant::options(),
            Some(executable_variant),
            M::SelectExecutableVariant,
        );
        let add_button_action =
            (!self.new_executable_name.is_empty()).then_some(M::GenerateExectuable);

        let edit_name =
            widget::text_input("-Name-", &self.new_executable_name).on_input(M::EditExecutableName);

        widget::column![
            widget::text("Settings"),
            widget::row![
                widget::button("Add").on_press_maybe(add_button_action),
                edit_name,
            ]
            .spacing(5),
            widget::column![
                widget::text("Executables"),
                widget::horizontal_rule(2),
                widget::scrollable(executables),
                widget::horizontal_rule(2)
            ],
            widget::row![widget::button("Remove").on_press_maybe(remove_button_action)].spacing(5),
            widget::row![
                widget::button("Pick Executable").on_press_maybe(pick_binary_action),
                widget::text!("{:?}", executable_bin),
            ]
            .spacing(5),
            widget::row![
                widget::button("Pick Assets").on_press_maybe(pick_assets_action),
                widget::text!("{:?}", executable_assets),
            ]
            .spacing(5),
            pick_variant,
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
