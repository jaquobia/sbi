use std::path::PathBuf;

use iced::{
    alignment::Vertical,
    widget::{self, center, container, mouse_area, opaque},
    Element,
    Length::{self, Fill},
    Padding, Task,
};

use crate::{
    config::{self, write_config_to_disk, SBIConfig},
    executable::Executable,
    game_launcher::{self, SBILaunchStatus},
    menus::{
        duplicate_profile::{DuplicateData, DuplicateSubmenuData, DuplicateSubmenuMessage},
        rename_profile::{RenameSubmenuData, RenameSubmenuMessage},
        ConfigureProfileSubmenuData, ConfigureProfileSubmenuMessage, NewProfileSubmenu,
        NewProfileSubmenuMessage, SettingsSubmenuData, SettingsSubmenuMessage,
    },
    profile::{self, Profile, ProfileData, ProfileJson},
    SBIDirectories,
};

// Main Application

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SubMenu {
    NewProfile(NewProfileSubmenu),
    ConfigureProfile(ConfigureProfileSubmenuData),
    Settings(SettingsSubmenuData),
    RenameProfile(RenameSubmenuData),
    DuplicateProfile(DuplicateSubmenuData),
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    Dummy(()),
    FetchedProfiles(Vec<Profile>),
    FetchedConfig(SBIConfig),
    LaunchedGame(SBILaunchStatus),
    CreateProfile(ProfileJson),
    ModifyCurrentProfile(ProfileJson),
    RenameCurrentProfile(String),
    DuplicateCurrentProfile(DuplicateData),
    DeleteCurrentProfile,
    CreateExecutable(String, PathBuf, Option<PathBuf>),
    RemoveExecutable(String),
    SelectExecutable(String),
    ButtonSettingsPressed,
    ButtonConfigureProfilePressed,
    ButtonLaunchPressed,
    ButtonExitSubmenuPressed,
    ButtonNewProfilePressed,
    ButtonRenamePressed,
    ButtonDuplicatePressed,
    ToggleDebug(bool),
    ToggleCloseOnLaunch(bool),
    SelectProfile(usize),
    // Submenu messages
    NewProfileMessage(NewProfileSubmenuMessage),
    SettingsMessage(SettingsSubmenuMessage),
    ConfigureProfileMessage(ConfigureProfileSubmenuMessage),
    RenameProfileMessage(RenameSubmenuMessage),
    DuplicateProfileMessage(DuplicateSubmenuMessage),
}

#[derive(Debug, Clone)]
pub struct Application {
    dirs: SBIDirectories,
    profiles: Vec<Profile>,
    config: SBIConfig,
    debug: bool,
    submenu: Option<SubMenu>,
    selected_profile: Option<usize>,
}

impl Application {
    pub fn new(dirs: SBIDirectories) -> Self {
        Self {
            dirs,
            profiles: vec![],
            config: SBIConfig::default(),
            debug: false,
            submenu: None,
            selected_profile: None,
        }
    }
    pub fn executables(&self) -> &rustc_hash::FxHashMap<String, Executable> {
        &self.config.executables
    }
    pub fn config(&self) -> &SBIConfig {
        &self.config
    }
    pub fn current_profile(&self) -> Option<&Profile> {
        self.selected_profile
            .map(|p| self.profiles.get(p))
            .flatten()
    }
    pub fn current_profile_mut(&mut self) -> Option<&mut Profile> {
        self.selected_profile
            .map(|p| self.profiles.get_mut(p))
            .flatten()
    }

    fn find_profiles_with_invalidated_executables(
        &self,
    ) -> impl Iterator<Item = Profile> + use<'_> {
        self.profiles
            .iter()
            .filter(|p| {
                p.selected_executable()
                    .map(|exe| self.executables().get(exe).is_none())
                    .unwrap_or_default()
            })
            .cloned()
    }

    fn write_config_task(&self) -> Task<Message> {
        let config = self.config.clone();
        let dir = self.dirs().data().to_path_buf();
        Task::perform(config::write_config_to_disk(dir, config), |_| {
            Message::Dummy(())
        })
    }
    fn write_profile_task(&self, profile: Profile) -> Task<Message> {
        Task::perform(profile::write_profile(profile), |_| Message::Dummy(()))
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dummy(()) => Task::none(),
            Message::FetchedProfiles(profiles) => {
                log::info!("Fetched profiles using async tasks! ({})", profiles.len());
                self.profiles = profiles;
                self.selected_profile = None;
                let write_tasks = self
                    .find_profiles_with_invalidated_executables()
                    .map(|mut p| {
                        p.clear_selected_executable();
                        p
                    })
                    .map(|p| self.write_profile_task(p))
                    .collect::<Vec<_>>();
                // Remove selected profile. There's no gurantee the previously selected profile
                // will be in the same place nor still exist after a fetch.
                // Unlike a tui environment, the user can just easily
                // re-select a profile.

                let profiles_dir = self.dirs().profiles().to_path_buf();
                let vanilla_profile_dir = self.dirs().vanilla_storage().map(PathBuf::from);
                if write_tasks.len() > 0 {
                    Task::batch(write_tasks).chain(Task::perform(
                        profile::find_profiles(profiles_dir, vanilla_profile_dir),
                        Message::FetchedProfiles,
                    ))
                } else {
                    Task::none()
                }
            }
            Message::FetchedConfig(config) => {
                self.config = config;
                Task::none()
            }
            Message::LaunchedGame(status) => {
                log::info!("Launched Game: {status:?}");
                Task::none()
            }
            Message::CreateProfile(profile) => {
                log::info!(
                    "Creating new profile - {} : {:?}",
                    profile.name,
                    profile.collection_id
                );
                let profiles_dir = self.dirs().profiles().to_path_buf();
                let maybe_vanilla_profile_dir = self.dirs().vanilla_storage().map(PathBuf::from);
                if let None = self.submenu.take() {
                    log::error!("Tried to create a profile while not in a create-profile screen!");
                }
                Task::perform(
                    crate::profile::create_profile_then_find_list(
                        profile,
                        profiles_dir,
                        maybe_vanilla_profile_dir,
                    ),
                    Message::FetchedProfiles,
                )
            }
            Message::ModifyCurrentProfile(json) => {
                log::info!("Modifying current profile...");
                if let Some(profile) = self.current_profile_mut() {
                    profile.set_json(json);
                    Task::perform(profile::write_profile(profile.clone()), |_| {
                        Message::Dummy(())
                    })
                } else {
                    log::error!("Trying to write data without a selected profile!!");
                    Task::none()
                }
            }
            Message::RenameCurrentProfile(name) => {
                if let Some(profile) = self.current_profile_mut() {
                    if let Some(json) = profile.json_mut() {
                        json.name = name;
                        Task::perform(profile::write_profile(profile.clone()), |_| {
                            Message::Dummy(())
                        })
                    } else {
                        // non-vanilla/default profiles do not have json
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::DuplicateCurrentProfile(data) => {
                if let Some(current_profile) = self.current_profile().cloned() {
                    let profiles_dir = self.dirs().profiles().to_path_buf();
                    let maybe_vanilla_profile_dir =
                        self.dirs().vanilla_storage().map(PathBuf::from);
                    Task::perform(
                        crate::profile::duplicate_profile_then_find_list(
                            current_profile,
                            data,
                            profiles_dir,
                            maybe_vanilla_profile_dir,
                        ),
                        Message::FetchedProfiles,
                    )
                } else {
                    Task::none()
                }
            }
            Message::DeleteCurrentProfile => {
                if let Some(profile) = self.current_profile() {
                    log::warn!("Deleting profile {}", profile.path().display());
                    let delete_path = profile.path().to_owned();
                    if let Some(p) = self.selected_profile.as_ref() {
                        self.profiles.remove(*p);
                    }
                    Task::perform(tokio::fs::remove_dir_all(delete_path), |_| {
                        Message::Dummy(())
                    })
                } else {
                    log::error!(
                        "Attempting to delete a profile without having a profile selected!!"
                    );
                    Task::none()
                }
            }
            Message::CreateExecutable(name, path, assets) => {
                log::info!(
                    "Creating executable: {}\n\tPath: {}\n\tAssets: {:?}",
                    name,
                    path.display(),
                    assets
                );
                self.config
                    .executables
                    .insert(name, Executable { bin: path, assets });
                self.write_config_task()
            }
            Message::RemoveExecutable(name) => {
                self.config.executables.remove(&name);
                let _ = self
                    .config
                    .default_executable
                    .take_if(|e| e.as_str().eq(name.as_str()));
                let profile_write_tasks = self
                    .find_profiles_with_invalidated_executables()
                    .map(|mut p| {
                        p.clear_selected_executable();
                        p
                    })
                    .map(|p| self.write_profile_task(p));
                Task::batch(profile_write_tasks).chain(self.write_config_task())
            }
            Message::ToggleDebug(state) => {
                log::info!("Toggling debug: {}", state);
                self.debug = state;
                Task::none()
            }
            Message::ToggleCloseOnLaunch(state) => {
                self.config.close_on_launch = state;
                self.write_config_task()
            }
            Message::SelectExecutable(executable) => {
                log::info!("Selecting executable: {}", executable);
                self.config.default_executable = Some(executable.clone());
                if let Some(profile) = self.current_profile_mut() {
                    if let Some(json) = profile.json_mut() {
                        json.selected_executable = Some(executable);
                    }
                    let profile = profile.clone();
                    self.write_profile_task(profile)
                } else {
                    Task::none()
                }
            }
            Message::ButtonSettingsPressed => {
                log::info!("Settings was pressed");
                self.submenu = Some(SubMenu::Settings(SettingsSubmenuData::new()));
                Task::none()
            }
            Message::ButtonConfigureProfilePressed => {
                log::info!("Configure Profile was pressed");
                if let Some(profile) = self
                    .selected_profile
                    .map(|p| self.profiles.get(p))
                    .flatten()
                    .map(|p| p.json())
                    .flatten()
                {
                    self.submenu = Some(SubMenu::ConfigureProfile(
                        ConfigureProfileSubmenuData::new(profile),
                    ));
                } else {
                    log::error!("Opened Configure Profile menu without a valid profile selected!!");
                }
                Task::none()
            }
            Message::ButtonExitSubmenuPressed => {
                log::info!("Back...");
                self.submenu = None;
                Task::none()
            }
            Message::ButtonLaunchPressed => {
                let profile = self
                    .selected_profile
                    .and_then(|p| self.profiles.get(p))
                    .cloned()
                    .expect("No profile selected?!");
                let executable = profile
                    .selected_executable()
                    .and_then(|name| self.executables().get(name))
                    .cloned()
                    .expect("NEED_EXECUTABLE_SELECTED_TO_LAUNCH_GAME");
                let vanilla_assets = self.dirs().vanilla_assets().to_path_buf();
                let vanilla_mods = self.dirs().vanilla_mods().map(|p| p.to_path_buf());
                let launch_settings = game_launcher::SBILaunchSettings { close_on_launch: self.config.close_on_launch };
                log::info!("Launching {} with {:?}", profile.name(), executable);
                Task::perform(
                    game_launcher::launch_game(executable, profile, vanilla_mods, vanilla_assets, launch_settings),
                    Message::LaunchedGame,
                )
            }
            Message::ButtonNewProfilePressed => {
                log::info!("New profile empty");
                self.submenu = Some(SubMenu::NewProfile(NewProfileSubmenu::new()));
                Task::none()
            }
            Message::ButtonRenamePressed => {
                log::info!("Rename profile");
                if let Some(profile) = self.current_profile() {
                    self.submenu = Some(SubMenu::RenameProfile(RenameSubmenuData::new(
                        profile.name(),
                    )));
                }
                Task::none()
            }
            Message::ButtonDuplicatePressed => {
                log::info!("Duplicate profile");
                if let Some(profile) = self.current_profile() {
                    self.submenu = Some(SubMenu::DuplicateProfile(DuplicateSubmenuData::new(
                        profile.name(),
                    )));
                }
                Task::none()
            }
            Message::SelectProfile(i) => {
                match self.profiles.get(i) {
                    Some(name) => {
                        log::info!("Selecting profile {} - {:?}", i, name);
                        self.selected_profile = Some(i);
                    }
                    None => {
                        log::error!("Selected profile {i} is out of bounds of the profile list of length {}!", self.profiles.len());
                    }
                }
                Task::none()
            }
            Message::NewProfileMessage(m) => {
                if let Some(SubMenu::NewProfile(t)) = self.submenu.as_mut() {
                    t.update(m)
                } else {
                    log::error!("Error: Tried to send a NewProfile message while not in a valid NewProfile submenu");
                    Task::none()
                }
            }
            Message::SettingsMessage(m) => {
                if let Some(SubMenu::Settings(s)) = self.submenu.as_mut() {
                    s.update(m)
                } else {
                    Task::none()
                }
            }
            Message::ConfigureProfileMessage(m) => {
                if let Some(SubMenu::ConfigureProfile(s)) = self.submenu.as_mut() {
                    s.update(m)
                } else {
                    Task::none()
                }
            }
            Message::RenameProfileMessage(m) => {
                if let Some(SubMenu::RenameProfile(s)) = self.submenu.as_mut() {
                    s.update(m)
                } else {
                    Task::none()
                }
            }
            Message::DuplicateProfileMessage(m) => {
                if let Some(SubMenu::DuplicateProfile(s)) = self.submenu.as_mut() {
                    s.update(m)
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn dirs(&self) -> &SBIDirectories {
        &self.dirs
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::TokyoNight
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Settings Button
        let settings_button = widget::button("Settings").on_press(Message::ButtonSettingsPressed);

        // New Profile Button
        let new_profile_button =
            widget::button("New Profile").on_press(Message::ButtonNewProfilePressed);

        // Debug Checkbox
        let debug_checkbox = widget::checkbox("Debug", self.debug).on_toggle(Message::ToggleDebug);

        // Top Bar
        let controls = widget::row![settings_button, new_profile_button, debug_checkbox,]
            .spacing(5)
            .height(40)
            .align_y(Vertical::Center);

        let maybe_profile_controls = if let Some(profile) = self.current_profile() {
            let selected_executable: Option<String> =
                profile.selected_executable().map(|s| s.to_string());

            // Launch Button
            let launch_button_message = selected_executable
                .is_some()
                .then(|| Message::ButtonLaunchPressed);

            let launch_button = widget::button("Launch")
                .on_press_maybe(launch_button_message)
                .width(Length::Fill);

            // Configure Profile Button
            let configure_profile_buttton_message =
                (!profile.is_vanilla()).then(|| Message::ButtonConfigureProfilePressed);

            let configure_profile_button = widget::button("Configure Profile")
                .on_press_maybe(configure_profile_buttton_message)
                .width(Length::Fill);

            // Executable Picker
            let executables = Vec::from_iter(self.executables().keys().cloned());
            let executable_picker = widget::pick_list(
                executables,
                selected_executable.clone(),
                Message::SelectExecutable,
            )
            .placeholder("Select an executable...");

            // Rename button
            let rename_profile_button =
                widget::button("Rename").on_press(Message::ButtonRenamePressed);
            // Duplicate button
            let duplicate_profile_button =
                widget::button("Duplicate").on_press(Message::ButtonDuplicatePressed);

            // Profile Configuration Panel
            let profile_controls = widget::column![
                launch_button,
                configure_profile_button,
                executable_picker,
                rename_profile_button,
                duplicate_profile_button
            ]
            .width(250)
            .spacing(3)
            .padding(Padding::new(5.0));

            self.selected_profile.map(|_i| profile_controls)
        } else {
            None
        };

        // Combine Profiles List, Profile Controls, Top Bar, and Popup Submenus
        let body = widget::row![self.view_select_profile()].push_maybe(maybe_profile_controls);
        let content = widget::column![controls, body,].padding(5);
        let popup = self.submenu.as_ref().map(|m| {
            Self::view_submenu(match m {
                SubMenu::NewProfile(m) => m.view(),
                SubMenu::ConfigureProfile(m) => m.view(&self),
                SubMenu::Settings(m) => m.view(&self).map(|m| m.into()),
                SubMenu::RenameProfile(m) => m.view(&self).map(|m| m.into()),
                SubMenu::DuplicateProfile(m) => m.view(&self).map(|m| m.into()),
            })
        });
        let stacked_content = widget::stack(std::iter::once(content.into())).push_maybe(popup);

        // Enable Debug Overlay + return
        let content: Element<'_, Message> = stacked_content.into();
        if self.debug {
            content.explain(iced::Color::WHITE)
        } else {
            content
        }
    }

    fn view_select_profile(&self) -> Element<'_, Message> {
        let profile_to_widget = |(i, p)| -> (usize, Element<'_, Message>) {
            let text_color = self
                .selected_profile
                .is_some_and(|p_i| p_i == i)
                .then(|| iced::Color::from_rgba(0.3, 0.7, 0.2, 1.0));
            let raw_text = widget::text!("{}", p)
                .width(Fill)
                .color_maybe(text_color)
                .size(20);
            let text = widget::column![raw_text, widget::horizontal_rule(2),];
            (
                i,
                mouse_area(text).on_press(Message::SelectProfile(i)).into(),
            )
        };
        let profiles = self.profiles.iter().map(Profile::name);
        let profiles = widget::keyed_column(profiles.enumerate().map(profile_to_widget))
            .width(Length::Fill)
            .align_items(iced::Alignment::Start)
            .spacing(8);
        let scrolling_profiles = widget::scrollable(profiles).height(Length::Fill).spacing(3);
        widget::container(scrolling_profiles)
            .width(Length::FillPortion(4))
            .into()
    }

    /// Preconfigured method of drawing containers in pop-ups. Can dead-lock the user if
    /// no method of closing the popup is provided.
    fn view_submenu(content_in: Element<'_, Message>) -> Element<'_, Message> {
        let popup_style = |theme: &iced::Theme| -> container::Style {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: iced::border::rounded(4),
                ..container::Style::default()
            }
        };
        // Dyanmically size the popup to be 50% of the window's width and height
        let inner_popup = widget::column![
            widget::vertical_space(),
            widget::row![
                widget::horizontal_space(),
                opaque(
                    container(content_in)
                        .width(Length::FillPortion(2))
                        .height(Length::FillPortion(2))
                        .style(popup_style)
                ),
                widget::horizontal_space(),
            ],
            widget::vertical_space(),
        ];
        // Fade the lower layer and intercept mouse inputs, center the popup
        opaque(
            mouse_area(
                center(
                    // opaque(
                    inner_popup, // )
                )
                .style(|_theme| container::Style {
                    background: Some(
                        iced::Color {
                            a: 0.8,
                            ..iced::Color::BLACK
                        }
                        .into(),
                    ),
                    ..Default::default()
                }),
            )
            .on_press(Message::ButtonExitSubmenuPressed),
        )
        // .explain(iced::Color::from_rgb(1.0, 0.5, 0.0))
    }
}
