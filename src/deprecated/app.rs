use std::{
    cell::RefCell,
    fs,
    io::stdout,
    ops::Rem,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    game_launcher,
    json::{InstanceDataJson, SBIConfig, SBILaunchMessageJson},
    spawn_sbi_service, tui,
    instance::{Instance, ModifyInstance},
    workshop_downloader, STARBOUND_BOOT_CONFIG_NAME,
};
use anyhow::{anyhow, Result};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use crate::
    tui::ui::popups::{
        confirmation::ConfirmationPopup, list_select::ListSelectPopup,
        modify_instance_executable::ModifyInstancePopup, new_instance::NewInstancePopup,
        rename_instance::RenamePopup, BoxedConsumablePopup, ConsumablePopup,
    }
;
use directories::ProjectDirs;


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
    pub popup: Option<RefCell<BoxedConsumablePopup<AppMessage>>>,
    pub instances: Vec<Instance>,
    pub instance_index: usize,
    proj_dirs: ProjectDirs,
    sender: UnboundedSender<SBILaunchMessageJson>,
    running_task: Option<tokio::task::JoinHandle<()>>,
    pub config: SBIConfig,
    should_quit: bool,
}

impl AppSBI {
    /// Run app and enter alternate TUI screen until app is finished
    pub async fn run(proj_dirs: ProjectDirs) -> Result<()> {
        let sender = spawn_sbi_service().await?;
        let mut app = AppSBI::new(proj_dirs, sender);
        app.update_instances()?;

        tui::setup()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut render_storage = tui::RenderStorage::new();

        let mut time_last = Instant::now();

        while !app.should_quit() {
            let mut maybe_message = handle_events(&mut app)?;
            while let Some(message) = maybe_message {
                maybe_message = app.handle_message(message)?;
            }
            render_storage.draw(&app, &mut terminal)?;

            let fixed_dt = Duration::from_secs_f64(1.0 / 60.0);
            while (Instant::now() - time_last) > fixed_dt {
                time_last += fixed_dt;
                render_storage.fixed_update();
            }
        }

        tui::tear_down()?;
        Ok(())
    }

    /// Create a new app from the project directory struct
    pub fn new(dirs: ProjectDirs, sender: UnboundedSender<SBILaunchMessageJson>) -> Self {
        let config = crate::core::load_or_generate_config(dirs.data_dir());
        Self {
            popup: None,
            instances: Vec::new(),
            instance_index: 0,
            proj_dirs: dirs,
            sender,
            running_task: None,
            config,
            should_quit: false,
        }
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
        // The instance we want to find again
        let current_instance_name = self
            .get_instance_current()
            .ok()
            .map(|i| i.name().to_string());
        let instances_dir = self.instances_dir()?;
        let mut instances = crate::core::parse_instance_paths_to_json(&crate::core::get_instance_json_paths(&instances_dir)?);
        instances.sort_by(|i1, i2| i1.name().cmp(i2.name()));

        self.set_instances(instances);
        if let Some(index) = current_instance_name
            .and_then(|name| self.instances.iter().position(|i| name.eq(i.name())))
        {
            self.instance_index = index;
        }
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
    /// Scroll the instance index up (visually) by subtracting 1, or wrapping at 0
    pub fn scroll_instances_up(&mut self) {
        if self.instance_index == 0 {
            self.instance_index = self.instances.len().saturating_sub(1);
        } else {
            self.instance_index = self.instance_index.saturating_sub(1);
        }
    }
    /// Scroll the instance index down (visually) by adding 1, or wrapping at the last element
    pub fn scroll_instances_down(&mut self) {
        if !self.instances.is_empty() {
            self.instance_index = self
                .instance_index
                .saturating_add(1)
                .rem(self.instances.len());
        }
    }

    pub fn is_task_running(&self) -> bool {
        self.running_task
            .as_ref()
            .is_some_and(|task| !task.is_finished())
    }

    pub fn open_popup(&mut self, popup: BoxedConsumablePopup<AppMessage>) {
        let _ = self.popup.insert(RefCell::new(popup));
    }
    pub fn close_popup(&mut self) -> Option<BoxedConsumablePopup<AppMessage>> {
        self.popup.take().map(|v| v.into_inner())
    }
    pub fn create_instance(&mut self, instance_data_json: InstanceDataJson) -> Result<()> {
        let instances_dir = self.instances_dir()?;
        crate::core::create_instance(&instances_dir, instance_data_json, &self.config)?;
        self.update_instances()?;
        Ok(())
    }
    pub fn delete_current_instance(&mut self) -> Result<()> {
        if let Ok(instance) = self.get_instance_current() {
            crate::core::delete_instance(instance)?;
            self.update_instances()?;
            self.scroll_instances_up();
        }
        Ok(())
    }
    pub fn launch_instance_cli(&mut self) -> Result<()> {
        let instance = self.get_instance_current()?;
        let instance_dir = instance.folder_path();
        let executable_name: String = instance
            .executable()
            .as_ref()
            .unwrap_or(&self.config.default_executable)
            .to_owned();
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
            .unwrap_or(exec_parent_path.clone());

        game_launcher::launch_instance_cli(&executable_path, instance_dir, Some(&sb_ld_path))
    }
    pub fn launch_instance_steam(&mut self) -> Result<()> {
        let (executable_name, bootconfig, instance_path) = {
            let instance = self.get_instance_current()?;
            let executable_name = instance
                .executable()
                .as_ref()
                .unwrap_or(&self.config.default_executable)
                .to_owned();
            let bootconfig = instance.folder_path().join(STARBOUND_BOOT_CONFIG_NAME);
            (executable_name, bootconfig, instance.folder_path())
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
        let launch_message = SBILaunchMessageJson {
            exececutable_path: executable_path,
            instance_path: Some(instance_path.to_path_buf()),
            ld_library_path: Some(sb_ld_path),
        };
        self.sender.send(launch_message)?;
        game_launcher::launch_instance_steam(Some(&bootconfig))
    }
    pub fn modify_instance(&mut self, modification: ModifyInstance) -> Result<()> {
        if let Ok(instance) = self.get_instance_current_mut() {
            crate::core::modify_instance(instance.clone(), modification, &self.config)?;
        }
        self.update_instances()
    }

    pub fn install_collection(&mut self) -> Result<()> {
        if let Ok(instance) = self.get_instance_current() {
            let force_install_dir = self.proj_dirs.data_dir().join("downloads");
            let instance_clone = instance.clone();
            self.running_task = Some(tokio::spawn(async move {
                let r = workshop_downloader::download_collection(instance_clone, force_install_dir)
                    .await;
                if let Result::Err(e) = r {
                    error!("Problem occured while downloading collection: {e}");
                }
            }));
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
            KeyCode::Up | KeyCode::Char('k') => {
                return Some(AppMessage::ScrollInstancesUp);
            }
            KeyCode::Down | KeyCode::Char('j') => {
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
    if app.is_task_running() {
        return Ok(None);
    }
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
