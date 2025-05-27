use std::{env, path::PathBuf, process::Command};

use eframe;
use egui::Button;
use logger::LogBuffer;
use org_roamers::server::ServerRuntime;
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
        viewport: egui::ViewportBuilder::default().with_inner_size([600., 800.]),
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
    runtime: Option<ServerRuntime>,
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
            runtime: None,
            logs,
        }
    }
}

impl OrgRoamersGUI {
    pub fn url(&self) -> anyhow::Result<String> {
        let port: usize = self.settings.port.parse()?;
        Ok(format!("{}:{}", self.settings.ip_addr, port))
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

            ui.separator();

            let button_label = if self.runtime.is_some() {
                "Stop Server"
            } else {
                "Start Server"
            };

            let button_width = ui.available_width();
            if ui
                .add_sized([button_width, 1.], Button::new(button_label))
                .clicked()
            {
                if self.runtime.is_some() {
                    let rt = self.runtime.take();
                    rt.unwrap().graceful_shutdown().unwrap();
                } else {
                    match start::start_server(&self) {
                        Ok(rt) => self.runtime = Some(rt),
                        Err(err) => {
                            print_gui_error(format!("Error starting server: {err:?}"));
                        }
                    }
                }
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
        let runtime = self.runtime.take();
        if let Some(rt) = runtime {
            rt.graceful_shutdown().unwrap();
        }
    }
}
