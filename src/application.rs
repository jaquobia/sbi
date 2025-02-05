use iced::Element;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Executable {
    XStarbound,
    OpenStarbound,
}

impl Executable {
    const ALL: [Executable; 2] = [
        Executable::OpenStarbound,
        Executable::XStarbound,
    ];
}

impl std::fmt::Display for Executable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Executable::XStarbound => "XStarbound",
            Executable::OpenStarbound => "OpenStarbound",
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct Application {
    executable_selection: Option<Executable>,
    debug: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectExecutable(Executable),
    ButtonSettingsPressed,
    ButtonLaunchPressed,
    ToggleDebug(bool),
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleDebug(state) => {
                self.debug = !self.debug;
            }
            Message::SelectExecutable(executable) => {
                println!("Selecting executable: {}", executable);
                self.executable_selection = Some(executable);
            }
            Message::ButtonSettingsPressed => {
                println!("Settings was pressed");
            }
            Message::ButtonLaunchPressed => {
                println!("Launching starbound...");
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Bottom Bar
        let settings_button_message = Some(Message::ButtonSettingsPressed);
        let settings_button = iced::widget::button("Settings").on_press_maybe(settings_button_message);
        let launch_button_message = Some(Message::ButtonLaunchPressed);
        let launch_button = iced::widget::button("Launch").on_press_maybe(launch_button_message);
        let executable_picker = iced::widget::pick_list(Executable::ALL, self.executable_selection.as_ref(), Message::SelectExecutable).placeholder("Select an executable...");
        let debug_checkbox = iced::widget::checkbox("Debug", self.debug).on_toggle(Message::ToggleDebug);
        let controls_right = iced::widget::row![
            executable_picker,
            launch_button,
        ];
        let controls = iced::widget::row![
            settings_button,
            debug_checkbox,
            iced::widget::horizontal_space(),
            controls_right,
        ];

        // Profiles Menu
        let profiles = iced::widget::container("Profiles");
        let content = iced::widget::column![
            profiles,
            iced::widget::vertical_space(),
            controls,
        ];

        // Total content
        let content: Element<'_, Message> = content.padding(5).into();
        if self.debug {
            content.explain(iced::Color::WHITE)
        } else {
            content
        }
    }
}
