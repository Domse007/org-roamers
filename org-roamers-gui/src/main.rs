#![windows_subsystem = "windows"]

use std::{env, path::PathBuf, process::Command};

use eframe;
use egui::{Button, IconData};
use logger::LogBuffer;
use rfd::FileDialog;
use settings::Settings;

mod logger;
mod settings;
mod start;

const LOG_ENTRIES: usize = 64;

fn main() {
    let log_buffer = LogBuffer::new();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(log_buffer.clone())
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    const NAME: &str = env!("CARGO_PKG_NAME");

    tracing::info!("Starting GUI...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(NAME)
            .with_drag_and_drop(true)
            .with_inner_size([600., 800.])
            .with_icon(OrgRoamersGUI::icon()),
        #[cfg(target_os = "linux")]
        // Set WM_CLASS to match the .desktop file
        window_builder: Some(Box::new(|builder| {
            builder.with_title(NAME).with_app_id(NAME)
        })),
        ..Default::default()
    };

    eframe::run_native(
        NAME,
        options,
        Box::new(|_cc| Ok(Box::new(OrgRoamersGUI::new(log_buffer)))),
    )
    .unwrap();
}

fn print_gui_error(err: String) {
    tracing::error!("---[ GUI ERROR ]----------------------------");
    tracing::error!("{err}");
    tracing::error!("--------------------------------------------");
}

#[cfg(target_os = "windows")]
fn settings_file() -> PathBuf {
    let mut path = start::config_path();
    path.push(".roamers-gui-settings.json");
    path
}

#[cfg(not(target_os = "windows"))]
fn settings_file() -> PathBuf {
    let mut path = env::home_dir().unwrap();
    path.push(".config/org-roamers/.roamers-gui-settings.json");
    path
}

struct OrgRoamersGUI {
    settings: Settings,
    logs: LogBuffer<LOG_ENTRIES>,
}

impl OrgRoamersGUI {
    fn new(logs: LogBuffer<LOG_ENTRIES>) -> Self {
        Self {
            settings: match Settings::read(settings_file()) {
                Ok(settings) => settings,
                Err(err) => {
                    print_gui_error(err.to_string());
                    Settings::default()
                }
            },
            logs,
        }
    }

    fn icon() -> IconData {
        const DATA: &[u8] = include_bytes!("../../web/public/org-roamers-gui.png");
        let image = image::load_from_memory(DATA).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        IconData {
            rgba: image.into_raw(),
            width,
            height,
        }
    }

    pub fn port(&self) -> anyhow::Result<u16> {
        self.settings.port.parse().map_err(Into::into)
    }

    pub fn host(&self) -> &str {
        self.settings.ip_addr.as_str()

    }

    pub fn url_with_protocol(&self) -> anyhow::Result<String> {
        let port: usize = self.settings.port.parse()?;
        Ok(format!("http://{}:{}", self.settings.ip_addr, port))
    }
}

impl eframe::App for OrgRoamersGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let ip_label = ui.add_sized([50., ui.available_height()], egui::Label::new("IP:"));
                ui.text_edit_singleline(&mut self.settings.ip_addr)
                    .labelled_by(ip_label.id);
            });
            ui.horizontal(|ui| {
                let port_label =
                    ui.add_sized([50., ui.available_height()], egui::Label::new("Port:"));
                ui.text_edit_singleline(&mut self.settings.port)
                    .labelled_by(port_label.id);
            });
            ui.horizontal(|ui| {
                let path_label =
                    ui.add_sized([50., ui.available_height()], egui::Label::new("Path:"));
                ui.text_edit_singleline(&mut self.settings.roam_path)
                    .labelled_by(path_label.id);
                if ui.button("Pick").clicked() {
                    match FileDialog::new().pick_folder() {
                        Some(dir) => self.settings.roam_path = dir.to_string_lossy().to_string(),
                        None => {}
                    }
                }
            });

            ui.checkbox(&mut self.settings.fs_watcher, "Enable file system watcher");

            ui.separator();

            // let button_label = if self.runtime.is_some() {
            //     "Stop Server"
            // } else {
            //     "Start Server"
            // };
            let button_label = "TODO: async stuff";

            let button_width = ui.available_width();
            if ui
                .add_sized([button_width, 1.], Button::new(button_label))
                .clicked()
            {
                todo!("Unimplemented because of switch to async.");
            }
            if ui
                .add_sized([button_width, 1.], Button::new("Open Website"))
                .clicked()
            {
                let _ = Command::new("xdg-open")
                    .arg(self.url_with_protocol().unwrap())
                    .status();
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add_sized(
                        [ui.available_width(), ui.available_height()],
                        egui::TextEdit::multiline(&mut self.logs.get_logs())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        let _ = self.settings.write(settings_file());
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.settings.write(settings_file());
    }
}
