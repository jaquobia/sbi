use std::path::PathBuf;

use directories::ProjectDirs;
use iced::{
    alignment::Vertical,
    widget::container,
    Element,
    Length::{self, Fill},
    Task,
};

use crate::profile::Profile;

// Executables

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Executable {
    XStarbound,
    OpenStarbound,
}

impl Executable {
    const ALL: [Executable; 2] = [Executable::OpenStarbound, Executable::XStarbound];
}

impl std::fmt::Display for Executable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Executable::XStarbound => "XStarbound",
            Executable::OpenStarbound => "OpenStarbound",
        })
    }
}

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
pub struct Application {
    directories: ProjectDirs,
    profiles: Vec<Profile>,
    debug: bool,
    submenu: Option<SubMenu>,
    executable_selection: Option<Executable>,
    selected_profile: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchedProfiles(Vec<Profile>),
    CreateProfile,
    SelectExecutable(Executable),
    ButtonSettingsPressed,
    ButtonLaunchPressed,
    ButtonExitSubmenuPressed,
    ButtonNewProfilePressed,
    ButtonNewProfileEmptyPressed,
    ToggleDebug(bool),
    SelectProfile(usize),
    NewProfileMessage(NewProfileSubmenuMessage),
}

impl Application {
    pub fn new(directories: ProjectDirs) -> Self {
        Self {
            directories,
            profiles: vec![],
            debug: false,
            submenu: None,
            executable_selection: None,
            selected_profile: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchedProfiles(profiles) => {
                println!("Fetched profiles using async tasks! ({})", profiles.len());
                self.profiles = profiles;
                // Remove selected profile. There's no gurantee the previously selected profile
                // will be in the same place nor still exist after a fetch.
                // Unlike a tui environment, the user can just easily
                // re-select a profile.
                self.selected_profile = None;
                // TODO: This may be deleted
                self.submenu = None;
                Task::none()
            }
            Message::CreateProfile => {
                if let Some(SubMenu::NewProfile(Some(t))) = self.submenu.as_ref() {
                    match t {
                        NewProfileType::Empty { name } => {
                            println!("Creating new empty profile - {name}");
                            // Make a new profile with just a name
                        }
                    }
                }
                self.submenu = None;
                let profiles_dir = self.profiles_directory();
                Task::perform(Self::find_profiles(profiles_dir), Message::FetchedProfiles)
            }
            Message::ToggleDebug(state) => {
                println!("Toggling debug: {}", state);
                self.debug = state;
                Task::none()
            }
            Message::SelectExecutable(executable) => {
                println!("Selecting executable: {}", executable);
                self.executable_selection = Some(executable);
                Task::none()
            }
            Message::ButtonSettingsPressed => {
                println!("Settings was pressed");
                self.submenu = Some(SubMenu::ConfigureProfile);
                Task::none()
            }
            Message::ButtonExitSubmenuPressed => {
                println!("Back...");
                self.submenu = None;
                Task::none()
            }
            Message::ButtonLaunchPressed => {
                println!("Launching starbound...");
                Task::none()
            }
            Message::ButtonNewProfilePressed => {
                println!("New profile pressed...");
                self.submenu = Some(SubMenu::NewProfile(None));
                Task::none()
            }
            Message::ButtonNewProfileEmptyPressed => {
                println!("New profile empty");
                self.submenu = Some(SubMenu::NewProfile(Some(NewProfileType::Empty {
                    name: String::from(""),
                })));
                Task::none()
            }
            Message::SelectProfile(i) => {
                match self.profiles.get(i) {
                    Some(name) => {
                        println!("Selecting profile {} - {:?}", i, name);
                        self.selected_profile = Some(i);
                    }
                    None => {
                        eprintln!("Selected profile {i} is out of bounds of the profile list of length {}!", self.profiles.len());
                    }
                }
                Task::none()
            }
            Message::NewProfileMessage(m) => {
                if let Some(SubMenu::NewProfile(Some(t))) = self.submenu.as_mut() {
                    t.update(m)
                } else {
                    eprintln!("Error: Tried to send a NewProfile message while not in a valid NewProfile submenu");
                    Task::none()
                }
            }
        }
    }

    pub fn profiles_directory(&self) -> PathBuf {
        self.directories.data_dir().join("profiles")
    }

    /// Returns a collection of all valid profiles in the profiles directory.
    /// A valid profile consists of a folder in the profiles directory which contains a valid json.
    pub async fn find_profiles(profiles_directory: std::path::PathBuf) -> Vec<Profile> {
        let paths = crate::profile::collect_profile_json_paths(&profiles_directory);
        match paths {
            Ok(paths) => crate::profile::parse_profile_paths_to_json(&paths),
            Err(e) => {
                eprintln!("Error gathering profiles: {e}");
                vec![]
            }
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::TokyoNight
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Bottom Bar
        let configure_profile_buttton_message = self
            .selected_profile
            .map(|_p_i| Message::ButtonSettingsPressed);
        let settings_button = iced::widget::button("Configure Profile")
            .on_press_maybe(configure_profile_buttton_message);

        let new_profile_button =
            iced::widget::button("New Profile").on_press(Message::ButtonNewProfilePressed);

        let launch_button_message = self
            .selected_profile
            .and(self.executable_selection)
            .map(|_p_i| Message::ButtonLaunchPressed);
        let launch_button = iced::widget::button("Launch").on_press_maybe(launch_button_message);
        let executable_picker = iced::widget::pick_list(
            Executable::ALL,
            self.executable_selection.as_ref(),
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
