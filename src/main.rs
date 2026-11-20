#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use enigo::{Button as EnigoButton, Direction, Enigo, Mouse, Settings as EnigoSettings};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const APP_TITLE: &str = "OP CLICKER 4.0";
const MAX_NORMAL_CPS: u8 = 30;
const MAX_TPS_CLICK_RATE: u8 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Profile {
    BasicWhite,
    MinecraftKeybind,
    MinecraftTps,
    RobloxAuto,
    RobloxKeybind,
}

impl Default for Profile {
    fn default() -> Self {
        Self::MinecraftTps
    }
}

impl Profile {
    fn label(self) -> &'static str {
        match self {
            Self::BasicWhite => "Basic White",
            Self::MinecraftKeybind => "Minecraft Keybind Clicker",
            Self::MinecraftTps => "Minecraft TPS Clicker  •  Recommended",
            Self::RobloxAuto => "Roblox Auto Clicker",
            Self::RobloxKeybind => "Roblox Keybind Clicker",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::BasicWhite => "A clean, plain-white clicker with the essential controls.",
            Self::MinecraftKeybind => "Hold or toggle a hotkey to send Minecraft mouse clicks.",
            Self::MinecraftTps => "Paces clicks against a Minecraft-style server tick rate.",
            Self::RobloxAuto => "A straightforward CPS clicker for Roblox experiences.",
            Self::RobloxKeybind => "Use a chosen keyboard hotkey to control Roblox clicking.",
        }
    }

    fn is_tps(self) -> bool {
        self == Self::MinecraftTps
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ActivationMode {
    Toggle,
    Hold,
}

impl Default for ActivationMode {
    fn default() -> Self {
        Self::Toggle
    }
}

impl ActivationMode {
    fn label(self) -> &'static str {
        match self {
            Self::Toggle => "Toggle hotkey",
            Self::Hold => "Hold hotkey",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ClickButton {
    Left,
    Right,
    Middle,
}

impl Default for ClickButton {
    fn default() -> Self {
        Self::Left
    }
}

impl ClickButton {
    fn label(self) -> &'static str {
        match self {
            Self::Left => "Left mouse button",
            Self::Right => "Right mouse button",
            Self::Middle => "Middle mouse button",
        }
    }

    fn enigo_button(self) -> EnigoButton {
        match self {
            Self::Left => EnigoButton::Left,
            Self::Right => EnigoButton::Right,
            Self::Middle => EnigoButton::Middle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Hotkey {
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    X,
    C,
    V,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self::F6
    }
}

impl Hotkey {
    fn label(self) -> &'static str {
        match self {
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::X => "X",
            Self::C => "C",
            Self::V => "V",
        }
    }

    fn keycode(self) -> Keycode {
        match self {
            Self::F6 => Keycode::F6,
            Self::F7 => Keycode::F7,
            Self::F8 => Keycode::F8,
            Self::F9 => Keycode::F9,
            Self::F10 => Keycode::F10,
            Self::F11 => Keycode::F11,
            Self::F12 => Keycode::F12,
            Self::X => Keycode::X,
            Self::C => Keycode::C,
            Self::V => Keycode::V,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    profile: Profile,
    activation: ActivationMode,
    hotkey: Hotkey,
    click_button: ClickButton,
    cps: u8,
    tps: u8,
    clicks_per_tick: u8,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            profile: Profile::MinecraftTps,
            activation: ActivationMode::Toggle,
            hotkey: Hotkey::F6,
            click_button: ClickButton::Left,
            cps: 12,
            tps: 20,
            clicks_per_tick: 1,
        }
    }
}

impl AppSettings {
    fn apply_profile_defaults(&mut self) {
        match self.profile {
            Profile::BasicWhite => {
                self.activation = ActivationMode::Toggle;
                self.cps = 10;
            }
            Profile::MinecraftKeybind => {
                self.activation = ActivationMode::Hold;
                self.cps = 10;
            }
            Profile::MinecraftTps => {
                self.activation = ActivationMode::Toggle;
                self.tps = 20;
                self.clicks_per_tick = 1;
            }
            Profile::RobloxAuto => {
                self.activation = ActivationMode::Toggle;
                self.cps = 12;
            }
            Profile::RobloxKeybind => {
                self.activation = ActivationMode::Hold;
                self.cps = 12;
            }
        }
    }

    fn sanitise(&mut self) {
        self.cps = self.cps.clamp(1, MAX_NORMAL_CPS);
        self.tps = self.tps.clamp(1, 20);
        self.clicks_per_tick = self.clicks_per_tick.clamp(1, 2);
    }
}

#[derive(Clone, Copy)]
struct WorkerSettings {
    activation: ActivationMode,
    hotkey: Hotkey,
    click_button: ClickButton,
    cps: u8,
    tps: u8,
    clicks_per_tick: u8,
    tps_profile: bool,
}

impl From<&AppSettings> for WorkerSettings {
    fn from(settings: &AppSettings) -> Self {
        Self {
            activation: settings.activation,
            hotkey: settings.hotkey,
            click_button: settings.click_button,
            cps: settings.cps,
            tps: settings.tps,
            clicks_per_tick: settings.clicks_per_tick,
            tps_profile: settings.profile.is_tps(),
        }
    }
}

struct RuntimeState {
    enabled: AtomicBool,
    running: AtomicBool,
    shutting_down: AtomicBool,
    click_count: AtomicU64,
    worker_message: Mutex<String>,
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            running: AtomicBool::new(false),
            shutting_down: AtomicBool::new(false),
            click_count: AtomicU64::new(0),
            worker_message: Mutex::new("Starting input worker…".to_owned()),
        }
    }

    fn set_message(&self, message: impl Into<String>) {
        if let Ok(mut current) = self.worker_message.lock() {
            *current = message.into();
        }
    }

    fn message(&self) -> String {
        self.worker_message
            .lock()
            .map(|value| value.clone())
            .unwrap_or_else(|_| "Input worker status unavailable".to_owned())
    }
}

struct OpClickerApp {
    settings: AppSettings,
    runtime: Arc<RuntimeState>,
    worker_settings: Arc<Mutex<WorkerSettings>>,
    worker: Option<JoinHandle<()>>,
    overlay_mode: bool,
}

impl OpClickerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.visuals.panel_fill = egui::Color32::WHITE;
        style.visuals.window_fill = egui::Color32::WHITE;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(246, 247, 249);
        cc.egui_ctx.set_style(style);

        let mut settings = load_settings();
        settings.sanitise();

        let runtime = Arc::new(RuntimeState::new());
        let worker_settings = Arc::new(Mutex::new(WorkerSettings::from(&settings)));
        let worker = start_worker(
            Arc::clone(&worker_settings),
            Arc::clone(&runtime),
        );

        Self {
            settings,
            runtime,
            worker_settings,
            worker,
            overlay_mode: false,
        }
    }

    fn sync_worker_settings(&self) {
        if let Ok(mut current) = self.worker_settings.lock() {
            *current = WorkerSettings::from(&self.settings);
        }
    }

    fn change_profile(&mut self, profile: Profile) {
        if self.settings.profile != profile {
            self.settings.profile = profile;
            self.settings.apply_profile_defaults();
            self.runtime.enabled.store(false, Ordering::SeqCst);
        }
    }

    fn current_cps(&self) -> u8 {
        if self.settings.profile.is_tps() {
            self.settings
                .tps
                .saturating_mul(self.settings.clicks_per_tick)
                .clamp(1, MAX_TPS_CLICK_RATE)
        } else {
            self.settings.cps.clamp(1, MAX_NORMAL_CPS)
        }
    }

    fn toggle_run(&self) {
        if self.settings.activation == ActivationMode::Toggle {
            let current = self.runtime.enabled.load(Ordering::SeqCst);
            self.runtime.enabled.store(!current, Ordering::SeqCst);
        }
    }

    fn show_profile_picker(&mut self, ui: &mut egui::Ui) {
        let selected = self.settings.profile;
        egui::ComboBox::from_label("Select variety")
            .selected_text(selected.label())
            .width(260.0)
            .show_ui(ui, |ui| {
                for profile in [
                    Profile::BasicWhite,
                    Profile::MinecraftKeybind,
                    Profile::MinecraftTps,
                    Profile::RobloxAuto,
                    Profile::RobloxKeybind,
                ] {
                    if ui
                        .selectable_label(self.settings.profile == profile, profile.label())
                        .clicked()
                    {
                        self.change_profile(profile);
                    }
                }
            });
    }

    fn show_hotkey_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_label("Start hotkey")
                .selected_text(self.settings.hotkey.label())
                .show_ui(ui, |ui| {
                    for key in [
                        Hotkey::F6,
                        Hotkey::F7,
                        Hotkey::F8,
                        Hotkey::F9,
                        Hotkey::F10,
                        Hotkey::F11,
                        Hotkey::F12,
                        Hotkey::X,
                        Hotkey::C,
                        Hotkey::V,
                    ] {
                        ui.selectable_value(&mut self.settings.hotkey, key, key.label());
                    }
                });

            egui::ComboBox::from_label("Activation")
                .selected_text(self.settings.activation.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.settings.activation,
                        ActivationMode::Toggle,
                        ActivationMode::Toggle.label(),
                    );
                    ui.selectable_value(
                        &mut self.settings.activation,
                        ActivationMode::Hold,
                        ActivationMode::Hold.label(),
                    );
                });

            egui::ComboBox::from_label("Click button")
                .selected_text(self.settings.click_button.label())
                .show_ui(ui, |ui| {
                    for button in [ClickButton::Left, ClickButton::Right, ClickButton::Middle] {
                        ui.selectable_value(
                            &mut self.settings.click_button,
                            button,
                            button.label(),
                        );
                    }
                });
        });
    }

    fn show_main(&mut self, ctx: &egui::Context) {
        let accent = profile_accent(self.settings.profile);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(
                        egui::RichText::new(APP_TITLE)
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::from_rgb(30, 35, 42)),
                    );
                    ui.label(
                        egui::RichText::new("RUST EDITION  •  GAME PROFILES  •  LIGHTWEIGHT")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(115, 123, 132)),
                    );
                });
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| self.show_profile_picker(ui),
                );
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);
            ui.colored_label(
                accent,
                egui::RichText::new(self.settings.profile.label()).strong().size(17.0),
            );
            ui.label(self.settings.profile.description());
            if self.settings.profile == Profile::MinecraftTps {
                ui.label(
                    egui::RichText::new("RECOMMENDED PROFILE")
                        .strong()
                        .size(10.0)
                        .color(egui::Color32::from_rgb(47, 143, 78)),
                );
            }

            ui.add_space(12.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CLICK ENGINE").strong().color(accent));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} CPS", self.current_cps()));
                    });
                });
                ui.add_space(5.0);
                if self.settings.profile.is_tps() {
                    ui.add(
                        egui::Slider::new(&mut self.settings.tps, 1..=20)
                            .text("Target TPS"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.settings.clicks_per_tick, 1..=2)
                            .text("Clicks per tick"),
                    );
                    ui.label("20 TPS = one 50 ms tick. This is a manual target; live server TPS is not detected.");
                } else {
                    ui.add(
                        egui::Slider::new(&mut self.settings.cps, 1..=MAX_NORMAL_CPS)
                            .text("Clicks per second (CPS)"),
                    );
                    ui.label("The click rate is capped at 30 CPS in this profile.");
                }
            });

            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("INPUT & HOTKEY").strong().color(accent));
                self.show_hotkey_controls(ui);
                ui.add_space(2.0);
                match self.settings.activation {
                    ActivationMode::Toggle => {
                        ui.label(format!(
                            "Press {} once to start, then press it again to stop.",
                            self.settings.hotkey.label()
                        ));
                    }
                    ActivationMode::Hold => {
                        ui.label(format!(
                            "Keep {} held down to click. Releasing it stops immediately.",
                            self.settings.hotkey.label()
                        ));
                    }
                }
            });

            ui.add_space(10.0);
            let running = self.runtime.running.load(Ordering::SeqCst);
            let clicks = self.runtime.click_count.load(Ordering::Relaxed);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("LIVE STATUS").strong().color(accent));
                    let status_color = if running {
                        egui::Color32::from_rgb(30, 150, 78)
                    } else {
                        egui::Color32::from_rgb(116, 124, 133)
                    };
                    ui.label(
                        egui::RichText::new(if running { "● RUNNING" } else { "● STOPPED" })
                            .strong()
                            .color(status_color),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Total clicks: {clicks}"));
                    });
                });
                ui.label(self.runtime.message());
            });

            ui.add_space(14.0);
            ui.horizontal_wrapped(|ui| {
                if self.settings.activation == ActivationMode::Toggle {
                    let running = self.runtime.enabled.load(Ordering::SeqCst);
                    let button_text = if running { "STOP CLICKING" } else { "START CLICKING" };
                    let fill = if running {
                        egui::Color32::from_rgb(190, 60, 60)
                    } else {
                        accent
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(button_text)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(fill)
                            .min_size(egui::vec2(170.0, 38.0)),
                        )
                        .clicked()
                    {
                        self.toggle_run();
                    }
                } else {
                    ui.label(
                        egui::RichText::new(format!(
                            "HOLD {} TO CLICK",
                            self.settings.hotkey.label()
                        ))
                        .strong()
                        .color(accent),
                    );
                }

                if ui.button("OPEN GAME OVERLAY").clicked() {
                    self.overlay_mode = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("OP CLICKER 4.0  •  BUILT IN RUST")
                        .size(10.0)
                        .color(egui::Color32::from_rgb(130, 137, 145)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("BASIC WHITE IS ALWAYS AVAILABLE")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(130, 137, 145)),
                    );
                });
            });
        });
    }

    fn show_overlay(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new("OP CLICKER 4.0").strong().size(19.0));
            ui.label(self.settings.profile.label());
            ui.separator();

            let running = self.runtime.running.load(Ordering::SeqCst);
            let color = if running {
                egui::Color32::from_rgb(28, 150, 78)
            } else {
                egui::Color32::from_rgb(120, 126, 134)
            };
            ui.colored_label(
                color,
                egui::RichText::new(if running { "● CLICKING" } else { "● STOPPED" }).strong(),
            );
            ui.label(format!("Rate: {} CPS", self.current_cps()));
            ui.label(format!("Hotkey: {}", self.settings.hotkey.label()));

            if self.settings.activation == ActivationMode::Toggle {
                let enabled = self.runtime.enabled.load(Ordering::SeqCst);
                if ui
                    .button(if enabled { "STOP" } else { "START" })
                    .clicked()
                {
                    self.toggle_run();
                }
            } else {
                ui.label(format!("Hold {} to run", self.settings.hotkey.label()));
            }

            if ui.button("CLOSE OVERLAY / FULL SETTINGS").clicked() {
                self.overlay_mode = false;
            }
        });
    }
}

impl eframe::App for OpClickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(60));

        let old_settings = self.settings.clone();
        let old_overlay = self.overlay_mode;

        if self.overlay_mode {
            self.show_overlay(ctx);
        } else {
            self.show_main(ctx);
        }

        if self.settings != old_settings {
            self.settings.sanitise();
            if self.settings.activation != old_settings.activation {
                self.runtime.enabled.store(false, Ordering::SeqCst);
            }
            self.sync_worker_settings();
            save_settings(&self.settings);
        }

        if self.overlay_mode != old_overlay {
            if self.overlay_mode {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(320.0, 220.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(340.0, 280.0)));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::Normal,
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(680.0, 520.0)));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(860.0, 650.0)));
            }
        }
    }
}

impl Drop for OpClickerApp {
    fn drop(&mut self) {
        self.runtime.shutting_down.store(true, Ordering::SeqCst);
        self.runtime.enabled.store(false, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        save_settings(&self.settings);
    }
}

fn start_worker(
    settings: Arc<Mutex<WorkerSettings>>,
    runtime: Arc<RuntimeState>,
) -> Option<JoinHandle<()>> {
    let spawn_status = Arc::clone(&runtime);
    let result = thread::Builder::new()
        .name("op-clicker-input".to_owned())
        .spawn(move || {
            let device = DeviceState::new();
            let mut enigo = match Enigo::new(&EnigoSettings::default()) {
                Ok(controller) => Some(controller),
                Err(error) => {
                    runtime.set_message(format!("Could not initialise mouse input: {error}"));
                    None
                }
            };

            if enigo.is_some() {
                runtime.set_message("Input ready. Keep your game under the mouse cursor.");
            }

            let mut was_hotkey_down = false;
            let mut was_active = false;
            let mut next_click = Instant::now();

            while !runtime.shutting_down.load(Ordering::Relaxed) {
                let config = match settings.lock() {
                    Ok(value) => *value,
                    Err(_) => {
                        runtime.set_message("Settings worker lock failed; clicker paused.");
                        runtime.running.store(false, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                };

                let keys = device.get_keys();
                let hotkey_down = keys.contains(&config.hotkey.keycode());

                if config.activation == ActivationMode::Toggle
                    && hotkey_down
                    && !was_hotkey_down
                {
                    runtime.enabled.fetch_xor(true, Ordering::SeqCst);
                }
                was_hotkey_down = hotkey_down;

                let active = match config.activation {
                    ActivationMode::Toggle => runtime.enabled.load(Ordering::SeqCst),
                    ActivationMode::Hold => hotkey_down,
                };
                if active && !was_active {
                    // A short grace period lets the user move the cursor from the UI to the game.
                    next_click = Instant::now() + Duration::from_millis(350);
                } else if !active {
                    next_click = Instant::now();
                }
                was_active = active;
                runtime.running.store(active && enigo.is_some(), Ordering::SeqCst);

                if active {
                    let cps = if config.tps_profile {
                        config
                            .tps
                            .clamp(1, 20)
                            .saturating_mul(config.clicks_per_tick.clamp(1, 2))
                            .clamp(1, MAX_TPS_CLICK_RATE)
                    } else {
                        config.cps.clamp(1, MAX_NORMAL_CPS)
                    };
                    let interval = Duration::from_nanos(1_000_000_000u64 / u64::from(cps));
                    let now = Instant::now();

                    if now >= next_click {
                        if let Some(controller) = enigo.as_mut() {
                            match controller.button(config.click_button.enigo_button(), Direction::Click) {
                                Ok(()) => {
                                    runtime.click_count.fetch_add(1, Ordering::Relaxed);
                                }
                                Err(error) => {
                                    runtime.set_message(format!(
                                        "Mouse input failed: {error}. Clicker paused."
                                    ));
                                    runtime.enabled.store(false, Ordering::SeqCst);
                                    enigo = None;
                                    runtime.running.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                        next_click = Instant::now() + interval;
                    }
                } else {
                    next_click = Instant::now();
                }

                // A short sleep keeps hotkey response quick without busy-spinning a CPU core.
                thread::sleep(Duration::from_millis(3));
            }

            runtime.running.store(false, Ordering::SeqCst);
        });

    match result {
        Ok(handle) => Some(handle),
        Err(error) => {
            spawn_status.set_message(format!("Could not start input worker: {error}"));
            None
        }
    }
}

fn profile_accent(profile: Profile) -> egui::Color32 {
    match profile {
        Profile::MinecraftKeybind | Profile::MinecraftTps => {
            egui::Color32::from_rgb(50, 137, 76)
        }
        Profile::RobloxAuto | Profile::RobloxKeybind => {
            egui::Color32::from_rgb(213, 67, 67)
        }
        Profile::BasicWhite => egui::Color32::from_rgb(55, 105, 165),
    }
}

fn settings_path() -> PathBuf {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.join("OPClicker4").join("settings.json")
}

fn load_settings() -> AppSettings {
    let path = settings_path();
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<AppSettings>(&bytes).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

fn save_settings(settings: &AppSettings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    if let Ok(json) = serde_json::to_vec_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 650.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(OpClickerApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_clamped_to_safe_click_rates() {
        let mut settings = AppSettings {
            cps: 255,
            tps: 255,
            clicks_per_tick: 255,
            ..AppSettings::default()
        };
        settings.sanitise();
        assert_eq!(settings.cps, MAX_NORMAL_CPS);
        assert_eq!(settings.tps, 20);
        assert_eq!(settings.clicks_per_tick, 2);
    }

    #[test]
    fn minecraft_tps_default_is_one_click_per_tick() {
        let settings = AppSettings::default();
        assert_eq!(settings.profile, Profile::MinecraftTps);
        assert_eq!(settings.tps, 20);
        assert_eq!(settings.clicks_per_tick, 1);
    }

    #[test]
    fn keybind_profiles_default_to_hold_activation() {
        let mut settings = AppSettings::default();
        settings.profile = Profile::RobloxKeybind;
        settings.apply_profile_defaults();
        assert_eq!(settings.activation, ActivationMode::Hold);
    }
}
