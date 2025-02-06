use directories::ProjectDirs;
use iced::{
    alignment::Vertical,
    Element,
    Length::{self, Fill},
};

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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SubMenu {
    #[default]
    SelectProfile,
    ConfigureProfile,
    Settings,
}

#[derive(Debug, Clone)]
pub struct Application {
    directories: ProjectDirs,
    debug: bool,
    submenu: SubMenu,
    executable_selection: Option<Executable>,
    selected_profile: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectExecutable(Executable),
    ButtonSettingsPressed,
    ButtonLaunchPressed,
    ButtonBackPressed,
    ToggleDebug(bool),
    SelectProfile(usize),
}

impl Application {
    pub fn new(directories: ProjectDirs) -> Self {
        Self {
            directories,
            debug: false,
            submenu: SubMenu::default(),
            executable_selection: None,
            selected_profile: None,
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleDebug(state) => {
                println!("Toggling debug: {}", state);
                self.debug = state;
            }
            Message::SelectExecutable(executable) => {
                println!("Selecting executable: {}", executable);
                self.executable_selection = Some(executable);
            }
            Message::ButtonSettingsPressed => {
                println!("Settings was pressed");
                self.submenu = SubMenu::ConfigureProfile;
            }
            Message::ButtonBackPressed => {
                println!("Back...");
                self.submenu = SubMenu::SelectProfile;
            }
            Message::ButtonLaunchPressed => {
                println!("Launching starbound...");
            }
            Message::SelectProfile(i) => {
                println!("Selecting profile {}", i);
                self.selected_profile = Some(i);
            }
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::TokyoNight
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Bottom Bar
        let (omni_button_label, omni_button_message) = match self.submenu {
            SubMenu::SelectProfile => (
                "Configure Profile",
                self.selected_profile
                    .map(|_p_i| Message::ButtonSettingsPressed),
            ),
            SubMenu::ConfigureProfile => ("Back", Some(Message::ButtonBackPressed)),
            SubMenu::Settings => ("Back", Some(Message::ButtonBackPressed)),
        };
        let settings_button =
            iced::widget::button(omni_button_label).on_press_maybe(omni_button_message);

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
            debug_checkbox,
            iced::widget::horizontal_space(),
            controls_right,
        ]
        .spacing(5)
        .height(40)
        .align_y(Vertical::Center);

        // Profiles Menu
        let body = match self.submenu {
            SubMenu::SelectProfile => self.view_select_profile(),
            SubMenu::ConfigureProfile => self.view_configure_profile(),
            SubMenu::Settings => self.view_settings(),
        };
        let content = iced::widget::column![body, controls,];

        // Total content
        let content: Element<'_, Message> = content.padding(5).into();
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
            let text = iced::widget::text!("{}", p)
                .width(Fill)
                .color_maybe(text_color);
            (
                i,
                iced::widget::mouse_area(text)
                    .on_press(Message::SelectProfile(i))
                    .into(),
            )
        };
        let profiles = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"];
        let profiles =
            iced::widget::keyed_column(profiles.iter().enumerate().map(profile_to_widget))
                .width(Length::Fill)
                .align_items(iced::Alignment::Start)
                .spacing(5);
        iced::widget::scrollable(profiles)
            .height(Length::Fill)
            .spacing(3)
            .into()
    }

    fn view_configure_profile(&self) -> Element<'_, Message> {
        iced::widget::row![].height(Length::Fill).into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        iced::widget::row![].height(Length::Fill).into()
    }
}
