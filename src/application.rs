use std::path::PathBuf;

use iced::{
    alignment::Vertical,
    widget::{self, center, container, mouse_area, opaque},
    Element,
    Length::{self, Fill},
    Padding, Task,
};

use crate::{
    config::{self, SBIConfig}, executable::Executable, game_launcher::{self, SBILaunchStatus}, menus::{NewProfileSubmenu, NewProfileSubmenuMessage, SettingsSubmenuData, SettingsSubmenuMessage}, profile::Profile, SBIDirectories
};


// Main Application

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SubMenu {
    NewProfile(NewProfileSubmenu),
    ConfigureProfile,
    Settings(SettingsSubmenuData),
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    Dummy(()),
    FetchedProfiles(Vec<Profile>),
    FetchedConfig(SBIConfig),
    LaunchedGame(SBILaunchStatus),
    CreateProfile,
    CreateExecutable(String, PathBuf, Option<PathBuf>),
    SelectExecutable(String),
    ButtonSettingsPressed,
    ButtonConfigureProfilePressed,
    ButtonLaunchPressed,
    ButtonExitSubmenuPressed,
    ButtonNewProfilePressed,
    ToggleDebug(bool),
    SelectProfile(usize),
    // Submenu messages
    NewProfileMessage(NewProfileSubmenuMessage),
    SettingsMessage(SettingsSubmenuMessage),
}

#[derive(Debug, Clone)]
pub struct Application {
    dirs: SBIDirectories,
    profiles: Vec<Profile>,
    executables: rustc_hash::FxHashMap<String, Executable>,
    debug: bool,
    submenu: Option<SubMenu>,
    selected_executable: Option<String>,
    selected_profile: Option<usize>,
}

impl Application {
    pub fn new(dirs: SBIDirectories) -> Self {
        let executables = rustc_hash::FxHashMap::default();
        Self {
            dirs,
            profiles: vec![],
            executables,
            debug: false,
            submenu: None,
            selected_executable: None,
            selected_profile: None,
        }
    }
    pub fn executables(&self) -> &rustc_hash::FxHashMap<String, Executable> {
        &self.executables
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dummy(()) => Task::none(),
            Message::FetchedProfiles(profiles) => {
                log::info!("Fetched profiles using async tasks! ({})", profiles.len());
                self.profiles = profiles;
                // Remove selected profile. There's no gurantee the previously selected profile
                // will be in the same place nor still exist after a fetch.
                // Unlike a tui environment, the user can just easily
                // re-select a profile.
                self.selected_profile = None;
                Task::none()
            }
            Message::FetchedConfig(config) => {
                self.executables = config.executables;
                Task::none()
            }
            Message::LaunchedGame(status) => {
                log::info!("Launched Game: {status:?}");
                Task::none()
            }
            Message::CreateProfile => {
                if let Some(SubMenu::NewProfile(t)) = self.submenu.as_ref() {
                    let profile = match t {
                        NewProfileSubmenu { name, collection_id } => {
                            let collection_id = (!collection_id.is_empty()).then_some(collection_id.clone());
                            log::info!("Creating new profile - {name} : {collection_id:?}");
                            // Make a new profile with just a name
                            crate::profile::ProfileJson {
                                name: name.clone(),
                                additional_assets: None,
                                collection_id,
                            }
                        }
                    };

                    self.submenu = None;
                    let profiles_dir = self.dirs().profiles().to_path_buf();
                    let maybe_vanilla_profile_dir =
                        self.dirs().vanilla_storage().map(PathBuf::from);
                    Task::perform(
                        crate::profile::create_profile_then_find_list(
                            profile,
                            profiles_dir,
                            maybe_vanilla_profile_dir,
                        ),
                        Message::FetchedProfiles,
                    )

                } else {
                    log::error!("Tried to create a profile while not in a create-profile screen!");
                    Task::none()
                }
            }
            Message::CreateExecutable(name, path, assets) => {
                log::info!("Creating executable: {} from {} with {:?}", name, path.display(), assets);
                let executables = {
                    let mut executables = self.executables.clone();
                    executables.insert(name, Executable { bin: path, assets });
                    executables
                };
                let config = SBIConfig { executables, ..Default::default() };
                let dir = self.dirs().data().to_path_buf();
                let write_task = Task::perform(config::write_config_to_disk(dir.to_owned(), config), |_| Message::Dummy(()));
                let read_task = Task::perform(config::load_config(dir), Message::FetchedConfig);
                write_task.chain(read_task)
            }
            Message::ToggleDebug(state) => {
                log::info!("Toggling debug: {}", state);
                self.debug = state;
                Task::none()
            }
            Message::SelectExecutable(executable) => {
                log::info!("Selecting executable: {}", executable);
                self.selected_executable = Some(executable);
                Task::none()
            }
            Message::ButtonSettingsPressed => {
                log::info!("Settings was pressed");
                self.submenu = Some(SubMenu::Settings(SettingsSubmenuData::new()));
                Task::none()
            }
            Message::ButtonConfigureProfilePressed => {
                log::info!("Configure Profile was pressed");
                self.submenu = Some(SubMenu::ConfigureProfile);
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
                let executable = self
                    .selected_executable
                    .as_ref()
                    .cloned()
                    .expect("No executable selected?!");
                let vanilla_assets = self.dirs().vanilla_assets().to_path_buf();
                log::info!("Launching {} with {:?}", profile.name(), executable);
                let executable = self
                    .executables
                    .get(&executable)
                    .cloned()
                    .expect("No executable matching name?!");
                Task::perform(
                    game_launcher::launch_game(executable, profile, vanilla_assets),
                    Message::LaunchedGame,
                )
            }
            Message::ButtonNewProfilePressed => {
                log::info!("New profile empty");
                self.submenu = Some(SubMenu::NewProfile(NewProfileSubmenu::new()));
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

        // Executable Picker
        let executables = Vec::from_iter(self.executables.keys().cloned());
        let executable_picker = widget::pick_list(
            executables,
            self.selected_executable.clone(),
            Message::SelectExecutable,
        )
        .placeholder("Select an executable...");

        // Debug Checkbox
        let debug_checkbox = widget::checkbox("Debug", self.debug).on_toggle(Message::ToggleDebug);

        // Top Bar
        let controls = widget::row![
            settings_button,
            new_profile_button,
            executable_picker,
            debug_checkbox,
        ]
        .spacing(5)
        .height(40)
        .align_y(Vertical::Center);

        // Launch Button
        let launch_button_message = self
            .selected_profile
            .and(self.selected_executable.as_ref())
            .map(|_p_i| Message::ButtonLaunchPressed);

        let launch_button = widget::button("Launch")
            .on_press_maybe(launch_button_message)
            .width(Length::Fill);

        // Configure Profile Button
        let configure_profile_buttton_message = self
            .selected_profile
            .and_then(|p_i| self.profiles.get(p_i))
            .filter(|p| !p.is_vanilla())
            .map(|_p_i| Message::ButtonConfigureProfilePressed);

        let configure_profile_button = widget::button("Configure Profile")
            .on_press_maybe(configure_profile_buttton_message)
            .width(Length::Fill);

        // Profile Configuration Panel
        let profile_controls = widget::column![launch_button, configure_profile_button,]
            .width(250)
            .spacing(3)
            .padding(Padding::new(5.0));

        let maybe_profile_controls = self.selected_profile.map(|_i| profile_controls);

        // Combine Profiles List, Profile Controls, Top Bar, and Popup Submenus
        let body = widget::row![self.view_select_profile()].push_maybe(maybe_profile_controls);
        let content = widget::column![controls, body,].padding(5);
        let popup = self.submenu.as_ref().map(|m| {
            Self::view_submenu(match m {
                SubMenu::NewProfile(t) => t.view(),
                SubMenu::ConfigureProfile => self.view_submenu_configure_profile(),
                SubMenu::Settings(m) => m.view(&self),
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

    fn view_submenu_configure_profile(&self) -> Element<'_, Message> {
        widget::column![
            widget::column![widget::text("Configuring Profile"),].spacing(8),
            widget::vertical_space(),
            widget::button("Close").on_press(Message::ButtonExitSubmenuPressed)
        ]
        .padding(5)
        .into()
    }
}
