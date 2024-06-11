use std::{
    cell::RefCell,
    fs,
    io::{self, stdout},
    path::{Path, PathBuf},
    process::Stdio,
};

use crate::json::{InstanceDataJson, SBIConfig};
use crate::{
    instance::{Instance, ModifyInstance},
    workshop_downloader, LOCAL_PIPE_NAME, SBI_CONFIG_JSON_NAME, STARBOUND_BOOT_CONFIG_NAME,
    STARBOUND_STEAM_ID,
};
use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::AsyncWriteExt;
use itertools::{Either, Itertools};
use log::{error, info};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use directories::ProjectDirs;
use ui::popups::{
    confirmation::ConfirmationPopup, list_select::ListSelectPopup,
    modify_instance_executable::ModifyInstancePopup, new_instance::NewInstancePopup,
    rename_instance::RenamePopup, BoxedConsumablePopup, ConsumablePopup,
};

use crate::{ui, INSTANCE_JSON_NAME};
/// Turns instance.json into Instance struct
fn parse_instance_paths_to_json(instance_json_paths: &[PathBuf]) -> Vec<Instance> {
    instance_json_paths
        .iter()
        .map(|ins_path| fs::read_to_string(ins_path).map(|str| (str, ins_path.clone())))
        .filter_map(Result::ok)
        .map(|(data, path)| {
            serde_json::from_str(&data).map(|data| Instance::from_json(data, &path))
        })
        .filter_map(Result::ok)
        .filter_map(Result::ok)
        .collect()
}

/// Returns an iterator of paths to the instance.json of each instance
fn get_instance_json_paths(instances_dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let instances = instances_dir
        .read_dir()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() || path.is_symlink())
        .map(|path| path.join(INSTANCE_JSON_NAME))
        .filter(|path| path.is_file())
        .collect();
    Ok(instances)
}

fn write_instance(instance: &Instance) -> Result<()> {
    let instance_path = instance.folder_path();
    fs::create_dir_all(instance_path)?;
    let instance_data = serde_json::to_string(instance.to_json())?;
    fs::write(instance_path.join("instance.json"), instance_data)?;
    Ok(())
}

pub enum AppMessage {
    Quit,
    LaunchInstanceCli,
    LaunchInstanceSteam,
    ScrollInstancesUp,
    ScrollInstancesDown,
    OpenPopup(Box<dyn ConsumablePopup<AppMessage>>),
    ClosePopup,
    ClosePopupNoOp,
    CreateInstance(InstanceDataJson),
    DeleteInstance,
    ModifyInstance(Vec<ModifyInstance>),
    InstallCollection,
}

pub struct AppSBI {
    popup: Option<RefCell<BoxedConsumablePopup<AppMessage>>>,
    instances: Vec<Instance>,
    instance_index: usize,
    default_executable: String,
    proj_dirs: ProjectDirs,
    sender: UnboundedSender<String>,
    config: SBIConfig,
    starbound_process: Option<std::thread::JoinHandle<()>>,
    should_quit: bool,
    debug: String,
}

impl AppSBI {
    /// Run app and enter alternate TUI screen until app is finished
    pub async fn run(proj_dirs: ProjectDirs) -> Result<()> {
        let (sender, reciver) = tokio::sync::mpsc::unbounded_channel::<String>();

        tokio::spawn(async {
            let mut reciver = reciver;

            let listener = match interprocess::local_socket::tokio::LocalSocketListener::bind(
                LOCAL_PIPE_NAME,
            ) {
                Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                    eprintln!("Error: could not start server because the socket file is occupied. Please check if {} is in use by another process and try again.", LOCAL_PIPE_NAME);
                    let var_name: Result<()> = Err(e.into());
                    return var_name;
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(x) => x,
            };

            loop {
                let conn = listener.accept().await?;
                let (_, mut writer) = conn.into_split();
                match reciver.try_recv() {
                    Ok(x) => {
                        writer.write_all(x.as_bytes()).await?;
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        });

        let mut app = AppSBI::new(proj_dirs, sender);
        app.update_instances()?;

        // Funcitons taken from Ratatui guide
        pub fn restore_tui() -> io::Result<()> {
            disable_raw_mode()?;
            crossterm::execute!(stdout(), LeaveAlternateScreen)?;
            Ok(())
        }

        pub fn init_panic_hook() {
            let original_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                // intentionally ignore errors here since we're already in a panic
                let _ = restore_tui();
                original_hook(panic_info);
            }));
        }

        init_panic_hook();

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        while !app.should_quit() {
            let mut maybe_message = handle_events(&mut app)?;
            while let Some(message) = maybe_message {
                maybe_message = app.handle_message(message)?;
            }
            terminal.draw(|frame| ui(frame, &app))?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    /// Create a new app from the project directory struct
    pub fn new(dirs: ProjectDirs, sender: UnboundedSender<String>) -> Self {
        let config = Self::load_or_generate_config(dirs.data_dir());
        Self {
            popup: None,
            instances: Vec::new(),
            instance_index: 0,
            default_executable: String::from("vanilla"),
            proj_dirs: dirs,
            sender,
            config,
            starbound_process: None,
            should_quit: false,
            debug: String::new(),
        }
    }
    /// Write the config struct to the json file
    fn write_config(data_dir: &Path, config: &SBIConfig) -> Result<()> {
        let contents = serde_json::to_string_pretty(config)?;
        fs::write(data_dir.join(SBI_CONFIG_JSON_NAME), contents)?;
        Ok(())
    }
    /// Read the config json file at data_dir and return the parsed SBIConfig struct
    fn read_json_from_string(data_dir: &Path) -> Result<SBIConfig> {
        let config_json_string = fs::read_to_string(data_dir.join(SBI_CONFIG_JSON_NAME))?;
        let config: SBIConfig = serde_json::from_str(&config_json_string)?;
        Ok(config)
    }
    /// Get config json from data directory or return
    /// default values if config is missing
    fn load_or_generate_config(data_dir: &Path) -> SBIConfig {
        Self::read_json_from_string(data_dir).unwrap_or_else(|_| {
            let config = SBIConfig {
                executables: rustc_hash::FxHashMap::default(),
            };
            // TODO: we probably should care
            let _we_dont_care = Self::write_config(data_dir, &config);
            config
        })
    }
    /// Set the application to a quit state
    /// Cannot undo this action
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    /// Return whether the app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    /// Get the Path to the instances directory
    pub fn instances_dir(&self) -> Result<PathBuf> {
        let data_dir = self.proj_dirs.data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }
        let instances_dir = data_dir.join("instances");
        if !instances_dir.exists() {
            fs::create_dir_all(instances_dir.clone())?;
        }
        Ok(instances_dir)
    }
    /// Get available instances from storage and set the instances array
    /// TODO: Ensure instance index is valid after update (Possible: Make new update_index fn?)
    pub fn update_instances(&mut self) -> Result<()> {
        let instances_dir = self.instances_dir()?;
        let instances = parse_instance_paths_to_json(&get_instance_json_paths(&instances_dir)?);
        self.set_instances(instances);
        Ok(())
    }
    /// Set the instances array
    pub fn set_instances(&mut self, instances: Vec<Instance>) {
        self.instances.clear();
        self.instances.extend(instances);
    }
    /// Get an instance at index
    pub fn get_instance(&self, index: usize) -> Result<&Instance> {
        let total_instances = self.instances.len();
        self.instances.get(index).ok_or(anyhow!(
            "No instance at index {} - total instances: {}",
            index,
            total_instances
        ))
    }
    pub fn get_instance_mut(&mut self, index: usize) -> Result<&mut Instance> {
        let total_instances = self.instances.len();
        self.instances.get_mut(index).ok_or(anyhow!(
            "No instance at index {} - total instances: {}",
            index,
            total_instances
        ))
    }
    /// Get an immutable reference to the instance at the instance index
    pub fn get_instance_current(&self) -> Result<&Instance> {
        self.get_instance(self.instance_index)
    }
    /// Get a mutable reference to the instance at the instance index
    fn get_instance_current_mut(&mut self) -> Result<&mut Instance> {
        self.get_instance_mut(self.instance_index)
    }
    /// Get a slice of all the Instances
    pub fn get_instances(&self) -> &[Instance] {
        &self.instances
    }
    /// Scroll the instance index up (subtracting 1), or stop at 0
    pub fn scroll_instances_up(&mut self) {
        self.instance_index = self.instance_index.saturating_sub(1);
    }
    /// Scroll the instance index down (adding 1), or stop at the last element
    pub fn scroll_instances_down(&mut self) {
        self.instance_index = self
            .instance_index
            .saturating_add(1)
            .min(self.instances.len().saturating_sub(1));
    }

    pub fn open_popup(&mut self, popup: BoxedConsumablePopup<AppMessage>) {
        let _ = self.popup.insert(RefCell::new(popup));
    }
    pub fn close_popup(&mut self) -> Option<BoxedConsumablePopup<AppMessage>> {
        self.popup.take().map(|v| v.into_inner())
    }
    pub fn create_instance(&mut self, instance_data_json: InstanceDataJson) -> Result<()> {
        let instances_dir = self.instances_dir()?;
        let mut instance_dir = instances_dir.join(instance_data_json.name.clone());
        // Folders with the same name does not exactly mean instances with the same name
        let mut i: usize = 1;
        while instance_dir.exists() {
            instance_dir = instances_dir.join(format!("{}_{}", &instance_data_json.name, i));
            i += 1;
        }
        let instance =
            Instance::from_json(instance_data_json, &instance_dir.join(INSTANCE_JSON_NAME))?;
        write_instance(&instance)?;
        self.generate_sbinit_config()?;
        self.update_instances()?;
        Ok(())
    }
    pub fn delete_current_instance(&mut self) -> Result<()> {
        let instance_path = self.get_instance_current()?.folder_path();
        fs::remove_dir_all(instance_path)?;
        self.update_instances()?;
        self.scroll_instances_up();
        Ok(())
    }
    pub fn launch_instance_cli(&mut self) -> Result<()> {
        let instance = self.get_instance_current()?;
        let executable_name: String = instance
            .executable()
            .as_ref()
            .unwrap_or(&self.default_executable)
            .to_owned();
        let executable = self
            .config
            .executables
            .get(&executable_name)
            .ok_or(anyhow!("Executable Name does not belong to an executable"))?;
        let executable_path = PathBuf::from(&executable.bin);

        // Calculate ld_path
        let os_ld_library_name = "LD_LIBRARY_PATH";
        let exec_parent_path = executable_path
            .parent()
            .ok_or(anyhow!("Executable path doesn't have a parent folder?!"))?
            .to_owned();
        let sb_ld_path = executable
            .ld_path
            .clone()
            .map(PathBuf::from)
            .unwrap_or(exec_parent_path.clone());
        let mut ld_paths = vec![sb_ld_path];
        if let Ok(system_ld_path) = std::env::var(os_ld_library_name) {
            ld_paths.extend(std::env::split_paths(&system_ld_path));
        };
        let new_ld_path_var = std::env::join_paths(ld_paths)?;

        info!(
            "Launching {} with ld_path: {:?}",
            executable_path.display(),
            new_ld_path_var
        );

        let mut command = tokio::process::Command::new(executable_path.clone());
        let instance_dir = instance.folder_path();
        command.current_dir(instance_dir);
        let bootconfig = instance_dir
            .join(STARBOUND_BOOT_CONFIG_NAME)
            .display()
            .to_string();
        command.env(os_ld_library_name, new_ld_path_var);
        command.args(["-bootconfig", &bootconfig]);
        command.stdout(Stdio::null()).stderr(Stdio::null()); // This little shit line caused me so
                                                             // many issues with zombie processes.
                                                             // Remember to unhook stdio for
                                                             // children you give up

        // This async thread is not necessary as we don't want to own children
        // but this also causes no harm
        tokio::task::spawn(async move { command.spawn()?.wait().await });
        Ok(())
    }
    pub fn launch_instance_steam(&mut self) -> Result<()> {
        let (executable_name, bootconfig) = {
            let instance = self.get_instance_current()?;
            let executable_name = instance
                .executable()
                .as_ref()
                .unwrap_or(&self.default_executable)
                .to_owned();
            let bootconfig = instance
                .folder_path()
                .join(STARBOUND_BOOT_CONFIG_NAME)
                .display()
                .to_string();
            (executable_name, bootconfig)
        };
        let (executable_path, sb_ld_path) = {
            let executable = self
                .config
                .executables
                .get(&executable_name)
                .ok_or(anyhow!("Executable Name does not belong to an executable"))?;
            let executable_path = PathBuf::from(&executable.bin);
            let exec_parent_path = executable_path
                .parent()
                .ok_or(anyhow!("Executable path doesn't have a parent folder?!"))?
                .to_owned();
            let sb_ld_path = executable
                .ld_path
                .clone()
                .map(PathBuf::from)
                .unwrap_or(exec_parent_path);
            (executable_path, sb_ld_path)
        };
        self.sender
            .send(format!("{}:{}", executable_path.display(), sb_ld_path.display()).to_string())?;
        let mut command = tokio::process::Command::new("steam");
        command.args(["-applaunch", STARBOUND_STEAM_ID, "-bootconfig", &bootconfig]);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn()?;
        tokio::task::spawn(async move { child.wait().await });
        Ok(())
    }
    pub fn generate_sbinit_config(&self) -> Result<()> {
        let instance = self.get_instance_current()?;
        let instance_folder = instance.folder_path();
        let sbinit_config_path = instance_folder.join(STARBOUND_BOOT_CONFIG_NAME);
        let executable = instance
            .executable()
            .as_ref()
            .and_then(|e| self.config.executables.get(e))
            .or_else(|| self.config.executables.get(&self.default_executable))
            .ok_or(anyhow!("Invalid Executable: {:?}", instance.executable()))?;
        let maybe_executable_assets = executable.custom_assets.as_ref();
        let mod_assets = instance_folder.join("mods");
        let vanilla_assets = self.proj_dirs.data_dir().join("assets");
        let maybe_additional_assets = instance.additional_assets();
        let storage_folder = instance_folder.join("storage");

        let mut assets = [vanilla_assets, mod_assets]
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect_vec();
        if let Some(executable_assets) = maybe_executable_assets {
            let executable_assets = PathBuf::from(&executable.bin)
                .parent()
                .unwrap()
                .join(executable_assets)
                .to_string_lossy()
                .to_string();
            assets.push(executable_assets);
        }
        if let Some(additional_assets) = maybe_additional_assets {
            // TODO: ~~apply instance_folder joining to the asset folder ONLY if its not a full path~~
            // Check if this works
            let additional_assets = additional_assets.iter().map(|f_name| {
                let p = PathBuf::from(f_name);
                if p.is_relative() {
                    instance_folder.join(p).to_string_lossy().to_string()
                } else {
                    f_name.to_owned()
                }
            });
            assets.extend(additional_assets);
        }
        let sbconfig_json = serde_json::json!({
            "assetDirectories": assets,
            "storageDirectory": storage_folder
        });

        let json_string = serde_json::to_string(&sbconfig_json)?;
        std::fs::write(sbinit_config_path, json_string).map_err(|e| anyhow!(e))
    }
    pub fn modify_instance(&mut self, modification: ModifyInstance) -> Result<()> {
        if let Ok(instance) = self.get_instance_current_mut() {
            instance.modify(modification);
            write_instance(instance)?;
            self.generate_sbinit_config()?;
        }
        self.update_instances()
    }

    pub fn install_collection(&mut self) -> Result<()> {
        if let Ok(instance) = self.get_instance_current() {
            let force_install_dir = self.proj_dirs.data_dir().join("downloads");
            let instance_clone = instance.clone();
            tokio::spawn(async move {
                let r = workshop_downloader::download_collection(
                    instance_clone,
                    force_install_dir,
                )
                .await;
                if let Result::Err(e) = r {
                    error!("{e}");
                }
            });
        }
        Ok(())
    }

    pub fn handle_message(&mut self, message: AppMessage) -> Result<Option<AppMessage>> {
        match message {
            AppMessage::Quit => {
                self.quit();
            }
            AppMessage::LaunchInstanceCli => {
                self.launch_instance_cli()?;
            }
            AppMessage::LaunchInstanceSteam => {
                self.launch_instance_steam()?;
            }
            AppMessage::ScrollInstancesUp => {
                self.scroll_instances_up();
            }
            AppMessage::ScrollInstancesDown => {
                self.scroll_instances_down();
            }
            AppMessage::OpenPopup(boxed_popup) => {
                self.open_popup(boxed_popup);
            }
            AppMessage::ClosePopup => {
                if let Some(mut popup) = self.close_popup() {
                    return Ok(popup.consume());
                }
            }
            AppMessage::ClosePopupNoOp => {
                let _ = self.close_popup();
            }
            AppMessage::CreateInstance(instance_json_data) => {
                self.create_instance(instance_json_data)?;
            }
            AppMessage::DeleteInstance => {
                self.delete_current_instance()?;
            }
            AppMessage::ModifyInstance(modifications) => {
                for modification in modifications {
                    self.modify_instance(modification)?;
                }
            }
            AppMessage::InstallCollection => {
                self.install_collection()?;
            }
        }
        Ok(None)
    }
}

fn handle_event_home(event: Event, app: &AppSBI) -> Option<AppMessage> {
    if let Event::Key(key) = event {
        if key.kind != event::KeyEventKind::Press {
            return None;
        }
        match key.code {
            KeyCode::Char('q') => {
                return Some(AppMessage::Quit);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(AppMessage::Quit);
            }
            KeyCode::Char('n') => {
                return Some(AppMessage::OpenPopup(Box::new(NewInstancePopup::new(
                    app.config.executables.keys().cloned().collect(),
                ))));
            }
            KeyCode::Char('m') => {
                if app.get_instance_current().is_err() {
                    return None;
                }
                return Some(AppMessage::OpenPopup(Box::new(
                    ListSelectPopup::new(vec![
                        (
                            "Rename",
                            AppMessage::OpenPopup(Box::new(RenamePopup::new())),
                        ),
                        ("Update", AppMessage::ClosePopupNoOp),
                        (
                            "Configure",
                            AppMessage::OpenPopup(Box::new(ModifyInstancePopup::new(
                                        app.get_instance_current().ok()?.clone(),
                                        app.config.executables.keys().cloned().collect(),
                            ))),
                        ),
                        ("(Re)Install Collection", AppMessage::InstallCollection),
                        (
                            "Delete",
                            AppMessage::OpenPopup(Box::new(ConfirmationPopup::new(
                                AppMessage::DeleteInstance,
                                String::from("Delete instance?"),
                            ))),
                        ),
                    ])
                    .set_block(
                        Block::default()
                            .title(String::from("Modify"))
                            .border_type(BorderType::Rounded)
                            .borders(Borders::ALL)
                            .bg(Color::Indexed(233)),
                    ),
                )));
            }
            KeyCode::Up | KeyCode::Char('j') => {
                return Some(AppMessage::ScrollInstancesUp);
            }
            KeyCode::Down | KeyCode::Char('k') => {
                return Some(AppMessage::ScrollInstancesDown);
            }
            KeyCode::Enter => {
                if app.get_instance_current().is_err() {
                    return None;
                }
                return Some(AppMessage::OpenPopup(Box::new(
                    ListSelectPopup::new(vec![
                        ("Run Client (Steam)", AppMessage::LaunchInstanceSteam),
                        ("Run Client (CLI)", AppMessage::LaunchInstanceCli),
                        ("Run Server", AppMessage::LaunchInstanceCli), // TODO
                        ("Cancel", AppMessage::ClosePopupNoOp),
                    ])
                    .set_block(
                        Block::default()
                            .title(String::from("Launch"))
                            .border_type(BorderType::Rounded)
                            .borders(Borders::ALL)
                            .bg(Color::Indexed(233)),
                    ),
                )));
            }
            _ => {}
        }
    }
    return None;
}

fn handle_events(app: &mut AppSBI) -> Result<Option<AppMessage>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        let event = event::read()?;
        // Unlike rendering, popups do not share event handling
        let message = if let Some(popup) = &app.popup {
            if let Event::Key(key) = &event {
                if key.code == KeyCode::Esc && key.kind == KeyEventKind::Press {
                    return Ok(Some(AppMessage::ClosePopupNoOp));
                }
            }
            let mut popup = popup.borrow_mut();
            popup.handle_event(&event)
        } else {
            handle_event_home(event, app)
        };
        return Ok(message);
    }

    Ok(None)
}

// fn handle_message(message: AppMessage, app: &mut AppSBI)
fn draw_keys(area: Rect, buffer: &mut Buffer, keys: &[(&str, &str)]) {
    let keybind_key_style = Style::new();
    let keybind_desc_style = Style::new();
    let keybind_separator_style = Style::new().fg(Color::Yellow);

    let separator = Span::styled("|", keybind_separator_style);
    let keys_last = keys.len() - 1;
    let key_spans = keys
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let key = Span::styled(format!(" {}:", key), keybind_key_style);
            let desc = Span::styled(format!("{} ", desc), keybind_desc_style);
            if i < keys_last {
                Either::Left([key, desc, separator.clone()].into_iter())
            } else {
                Either::Right([key, desc].into_iter())
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(Line::from(key_spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232))
        .render(area, buffer);
}

fn draw_home(area: Rect, buffer: &mut Buffer, app: &AppSBI) {
    use Constraint as C;
    let title_style = Style::new();
    let highlighted_instance_style = Style::new().bg(Color::White).fg(Color::Black);
    let home_border_style = Style::new().fg(Color::Green);
    let instance_list_style = Style::new().fg(Color::White);
    let instance_info_style = Style::new().fg(Color::White);

    // Draw borders and Title
    Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_type(BorderType::Thick)
        .title("SBI")
        .title_alignment(Alignment::Left)
        .title_style(title_style)
        .style(home_border_style)
        .bg(Color::Indexed(233))
        .render(area, buffer);
    {
        let current_instance = if let Ok(instance) = app.get_instance_current() {
            instance
        } else {
            return;
        };
        let executable = match current_instance.executable() {
            Some(executable) => {
                format!("{}", executable)
            }
            None => {
                format!("Default({})", app.default_executable)
            }
        };
        let line_1 = Line::from(vec![
            Span::styled("Name: ", instance_info_style),
            Span::styled(current_instance.name(), instance_info_style),
        ]);
        let line_2 = Line::from(vec![
            Span::styled("Executable: ", instance_info_style),
            Span::styled(executable, instance_info_style),
        ]);
        let lines = vec![line_1, line_2];
        let lines_count = lines.len();
        let text = Text::from(lines);

        let [area_instance_list, area_line_separator, area_instance_info, _] = ui::layout(
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
            Direction::Vertical,
            [
                C::Min(0),
                C::Length(1),
                C::Length(lines_count as u16),
                C::Length(0),
            ],
        );
        if !app.instances.is_empty() {
            let items = app.get_instances().iter().map(|ins| ins.name());
            let list = List::new(items)
                .style(instance_list_style)
                .highlight_style(highlighted_instance_style);
            let mut state = ListState::default().with_selected(Some(app.instance_index));
            StatefulWidget::render(list, area_instance_list, buffer, &mut state);
        }
        Block::new()
            .borders(Borders::TOP)
            .style(home_border_style)
            .render(area_line_separator, buffer);
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .render(area_instance_info, buffer);
    }
}

fn ui(frame: &mut Frame, app: &AppSBI) {
    let area = frame.size();
    let buffer = frame.buffer_mut();

    use Constraint as C;
    let [area_instances, area_debug, area_keybinds] = ui::layout(
        area,
        Direction::Vertical,
        [C::Min(0), C::Length(1), C::Length(1)],
    );
    draw_home(area_instances, buffer, app);

    Paragraph::new(Line::from(app.debug.to_owned())).render(area_debug, buffer);

    // Draw Status and Keybinds
    let keys = [
        ("Q", "Quit"),
        ("↑/j", "Up"),
        ("↓/k", "Down"),
        ("Enter", "Run Options"),
        ("n", "New Instance"),
        ("m", "Modify Instance"),
    ];
    draw_keys(area_keybinds, buffer, &keys);
    if let Some(popup) = &app.popup {
        popup.borrow().ui(buffer, area);
    }
}
