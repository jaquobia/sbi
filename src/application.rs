use std::path::PathBuf;

use iced::{
    alignment::Vertical,
    widget::container,
    Element,
    Length::{self, Fill},
    Task,
};

use crate::{config::SBIConfig, executable::Executable, game_launcher::{self, SBILaunchStatus}, profile::Profile, SBIDirectories};


// New Profile Submenu

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NewProfileSubmenuMessage {
    TextFieldEditName(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NewProfileType {
    Empty { name: String },
}

impl NewProfileType {
    fn update(&mut self, message: NewProfileSubmenuMessage) -> Task<Message> {
        match message {
            NewProfileSubmenuMessage::TextFieldEditName(s) => {
                match self {
                    Self::Empty { name } => {
                        *name = s;
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Empty { name } => {
                let create_button_message = (!name.is_empty()).then_some(Message::CreateProfile);
                iced::widget::column![
                    iced::widget::column![
                        iced::widget::text("New Profile - Empty"),
                        iced::widget::text_input("-Name-", name).on_input(|s| {
                            Message::NewProfileMessage(NewProfileSubmenuMessage::TextFieldEditName(
                                s,
                            ))
                        }),
                    ]
                    .spacing(8),
                    iced::widget::vertical_space(),
                    iced::widget::row![
                        iced::widget::button("Back").on_press(Message::ButtonNewProfilePressed),
                        iced::widget::horizontal_space(),
                        iced::widget::button("Create").on_press_maybe(create_button_message),
                    ]
                ]
                .padding(5)
                .into()
            }
        }
    }
}

// Main Application

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SubMenu {
    NewProfile(Option<NewProfileType>),
    ConfigureProfile,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchedProfiles(Vec<Profile>),
    FetchedConfig(SBIConfig),
    LaunchedGame(SBILaunchStatus),
    CreateProfile,
    SelectExecutable(String),
    ButtonSettingsPressed,
    ButtonLaunchPressed,
    ButtonExitSubmenuPressed,
    ButtonNewProfilePressed,
    ButtonNewProfileEmptyPressed,
    ToggleDebug(bool),
    SelectProfile(usize),
    #[allow(clippy::enum_variant_names)]
    NewProfileMessage(NewProfileSubmenuMessage),
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
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchedProfiles(profiles) => {
                log::info!("Fetched profiles using async tasks! ({})", profiles.len());
                self.profiles = profiles;
                // Remove selected profile. There's no gurantee the previously selected profile
                // will be in the same place nor still exist after a fetch.
                // Unlike a tui environment, the user can just easily
                // re-select a profile.
                self.selected_profile = None;
                Task::none()
            },
            Message::FetchedConfig(config) => {
                self.executables = config.executables;
                Task::none()
            },
            Message::LaunchedGame(status) => {
                log::info!("Launched Game: {status:?}");
                Task::none()
            },
            Message::CreateProfile => {
                if let Some(SubMenu::NewProfile(Some(t))) = self.submenu.as_ref() {
                    let profile = match t {
                        NewProfileType::Empty { name } => {
                            log::info!("Creating new empty profile - {name}");
                            // Make a new profile with just a name
                            crate::profile::ProfileJson { name: name.clone(), additional_assets: None, collection_id: None }
                        }
                    };

                    self.submenu = None;
                    let profiles_dir = self.dirs().profiles().to_path_buf();
                    let maybe_vanilla_profile_dir = self.dirs().vanilla_storage().map(PathBuf::from);
                    Task::perform(crate::profile::create_profile_then_find_list(profile, profiles_dir, maybe_vanilla_profile_dir), Message::FetchedProfiles)
                } else {
                    log::error!("Tried to create a profile while not in a create-profile screen!");
                    Task::none()
                }
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
                self.submenu = Some(SubMenu::ConfigureProfile);
                Task::none()
            }
            Message::ButtonExitSubmenuPressed => {
                log::info!("Back...");
                self.submenu = None;
                Task::none()
            }
            Message::ButtonLaunchPressed => {
                let profile = self.selected_profile.and_then(|p|self.profiles.get(p)).cloned().expect("No profile selected?!");
                let executable = self.selected_executable.as_ref().cloned().expect("No executable selected?!");
                let vanilla_assets = self.dirs().vanilla_assets().to_path_buf();
                log::info!("Launching {} with {:?}", profile.name(), executable);
                let executable = self.executables.get(&executable).cloned().expect("No executable matching name?!");
                Task::perform(game_launcher::launch_game(executable, profile, vanilla_assets), Message::LaunchedGame)
            }
            Message::ButtonNewProfilePressed => {
                log::info!("New profile pressed...");
                self.submenu = Some(SubMenu::NewProfile(None));
                Task::none()
            }
            Message::ButtonNewProfileEmptyPressed => {
                log::info!("New profile empty");
                self.submenu = Some(SubMenu::NewProfile(Some(NewProfileType::Empty {
                    name: String::from(""),
                })));
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
                if let Some(SubMenu::NewProfile(Some(t))) = self.submenu.as_mut() {
                    t.update(m)
                } else {
                    log::error!("Error: Tried to send a NewProfile message while not in a valid NewProfile submenu");
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
        // Bottom Bar
        let configure_profile_buttton_message = self
            .selected_profile
            .and_then(|p_i| self.profiles.get(p_i))
            .filter(|p| !p.is_vanilla() )
            .map(|_p_i| Message::ButtonSettingsPressed);
        let settings_button = iced::widget::button("Configure Profile")
            .on_press_maybe(configure_profile_buttton_message);

        let new_profile_button =
            iced::widget::button("New Profile").on_press(Message::ButtonNewProfilePressed);

        let launch_button_message = self
            .selected_profile
            .and(self.selected_executable.as_ref())
            .map(|_p_i| Message::ButtonLaunchPressed);
        let launch_button = iced::widget::button("Launch").on_press_maybe(launch_button_message);
        let executables = Vec::from_iter(self.executables.keys().cloned());
        let executable_picker = iced::widget::pick_list(
            executables,
            self.selected_executable.clone(),
            Message::SelectExecutable,
        )
        .placeholder("Select an executable...");
        let debug_checkbox =
            iced::widget::checkbox("Debug", self.debug).on_toggle(Message::ToggleDebug);

        let controls_right = iced::widget::row![executable_picker, launch_button,].spacing(5);
        let controls = iced::widget::row![
            settings_button,
            new_profile_button,
            debug_checkbox,
            iced::widget::horizontal_space(),
            controls_right,
        ]
        .spacing(5)
        .height(40)
        .align_y(Vertical::Center);

        // Profiles Menu
        let body = self.view_select_profile();
        let content = iced::widget::column![body, controls,].padding(5);
        let popup = self.submenu.as_ref().map(|m| {
            Self::view_submenu(match m {
                SubMenu::NewProfile(t) => match t {
                    None => self.view_submenu_new_profile(),
                    Some(t) => t.view(),
                },
                // SubMenu::NewProfileEmpty => self.view_submenu_new_profile_empty(),
                SubMenu::ConfigureProfile => self.view_submenu_configure_profile(),
                SubMenu::Settings => self.view_submenu_settings(),
            })
        });
        let stacked_content =
            iced::widget::stack(std::iter::once(content.into())).push_maybe(popup);

        // Total content
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
            let text = iced::widget::column![
                iced::widget::text!("{}", p)
                    .width(Fill)
                    .color_maybe(text_color),
                iced::widget::horizontal_rule(2),
            ];
            (
                i,
                iced::widget::mouse_area(text)
                    .on_press(Message::SelectProfile(i))
                    .into(),
            )
        };
        let profiles = self.profiles.iter().map(|p| p.name());
        let profiles = iced::widget::keyed_column(profiles.enumerate().map(profile_to_widget))
            .width(Length::Fill)
            .align_items(iced::Alignment::Start)
            .spacing(8);
        iced::widget::scrollable(profiles)
            .height(Length::Fill)
            .spacing(3)
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
        let inner_popup = iced::widget::column![
            iced::widget::vertical_space(),
            iced::widget::row![
                iced::widget::horizontal_space(),
                iced::widget::container(content_in)
                    .width(Length::FillPortion(2))
                    .height(Length::FillPortion(2))
                    .style(popup_style),
                iced::widget::horizontal_space(),
            ],
            iced::widget::vertical_space(),
        ];
        // Fade the lower layer and intercept mouse inputs, center the popup
        iced::widget::opaque(
            iced::widget::center(iced::widget::opaque(inner_popup)).style(|_theme| {
                container::Style {
                    background: Some(
                        iced::Color {
                            a: 0.8,
                            ..iced::Color::BLACK
                        }
                        .into(),
                    ),
                    ..Default::default()
                }
            }),
        )
        // .explain(iced::Color::from_rgb(1.0, 0.5, 0.0))
    }

    fn view_submenu_new_profile(&self) -> Element<'_, Message> {
        iced::widget::column![
            iced::widget::column![
                iced::widget::text("New Profile"),
                iced::widget::button("Empty").on_press(Message::ButtonNewProfileEmptyPressed),
                iced::widget::button("Vanilla (Steam)"),
                iced::widget::button("Collection (Steam)"),
            ]
            .spacing(8),
            iced::widget::vertical_space(),
            iced::widget::button("Close").on_press(Message::ButtonExitSubmenuPressed)
        ]
        .padding(5)
        .into()
    }

    fn view_submenu_configure_profile(&self) -> Element<'_, Message> {
        iced::widget::row![].into()
    }

    fn view_submenu_settings(&self) -> Element<'_, Message> {
        iced::widget::row![].into()
    }
}
