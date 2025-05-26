use std::process::Command;

use eframe;
use org_roamers::server::ServerRuntime;
use rfd::FileDialog;

mod start;

fn main() {
    const NAME: &str = env!("CARGO_PKG_NAME");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        NAME,
        options,
        Box::new(|_cc| Ok(Box::<OrgRoamersGUI>::default())),
    )
    .unwrap();
}

struct OrgRoamersGUI {
    ip_addr: String,
    port: String,
    roam_path: String,
    runtime: Option<ServerRuntime>,
}

impl Default for OrgRoamersGUI {
    fn default() -> Self {
        Self {
            ip_addr: "localhost".to_string(),
            port: "5000".to_string(),
            roam_path: "".to_string(),
            runtime: None,
        }
    }
}

impl OrgRoamersGUI {
    pub fn url(&self) -> anyhow::Result<String> {
        let port: usize = self.port.parse()?;
        Ok(format!("{}:{}", self.ip_addr, port))
    }

    pub fn url_with_protocol(&self) -> anyhow::Result<String> {
        let port: usize = self.port.parse()?;
        Ok(format!("http://{}:{}", self.ip_addr, port))
    }
}

impl eframe::App for OrgRoamersGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let ip_label = ui.label("IP:");
                ui.text_edit_singleline(&mut self.ip_addr)
                    .labelled_by(ip_label.id);
            });
            ui.horizontal(|ui| {
                let port_label = ui.label("Port:");
                ui.text_edit_singleline(&mut self.port)
                    .labelled_by(port_label.id);
            });
            ui.horizontal(|ui| {
                let path_label = ui.label("Path:");
                ui.text_edit_singleline(&mut self.roam_path)
                    .labelled_by(path_label.id);
                if ui.button("Pick").clicked() {
                    match FileDialog::new().pick_folder() {
                        Some(dir) => self.roam_path = dir.to_string_lossy().to_string(),
                        None => {}
                    }
                }
            });
            let button_label = if self.runtime.is_some() {
                "Stop Server"
            } else {
                "Start Server"
            };
            ui.horizontal(|ui| {
                if ui.button(button_label).clicked() {
                    if self.runtime.is_some() {
                        let rt = self.runtime.take();
                        rt.unwrap().graceful_shutdown().unwrap();
                    } else {
                        self.runtime = Some(start::start_server(&self).unwrap());
                    }
                }
                if ui.button("Open Website").clicked() {
                    let _ = Command::new("xdg-open")
                        .arg(self.url_with_protocol().unwrap())
                        .status();
                }
            });
        });
    }
}
