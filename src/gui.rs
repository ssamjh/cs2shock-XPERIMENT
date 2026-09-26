use std::{
    process,
    sync::{mpsc, Arc},
};

use eframe::icon_data::from_png_bytes;
use egui::{widgets::DragValue, Button, ViewportBuilder};
use log::{debug, info};
use tokio::sync::RwLock;

use crate::{
    config::{Config, ShockMode},
    openshock::{self, DiscoveredShocker},
};

pub async fn run(config: Arc<RwLock<Config>>) {
    let png_bytes = include_bytes!("../assets/icon.png");
    let viewport = ViewportBuilder::default()
        .with_inner_size([480.0, 680.0])
        .with_min_inner_size([360.0, 420.0])
        .with_resizable(true)
        .with_icon(Arc::new(
            from_png_bytes(png_bytes).expect("Failed to load icon"),
        ));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let changes = config.read().await.clone();
    let _ = eframe::run_native(
        "CS2 Shock",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(config, changes)))),
    );
}

struct MyApp {
    config: Arc<RwLock<Config>>,
    changes: Config,
    discovered: Vec<DiscoveredShocker>,
    discovery_rx: Option<mpsc::Receiver<Result<Vec<DiscoveredShocker>, String>>>,
    discovery_loading: bool,
    discovery_complete: bool,
    discovery_error: Option<String>,
    discovery_request: Option<(String, String)>,
    manual_shocker_id: String,
    logs_open: bool,
    save_error: Option<String>,
}

impl MyApp {
    fn new(config: Arc<RwLock<Config>>, changes: Config) -> Self {
        Self {
            config,
            changes,
            discovered: Vec::new(),
            discovery_rx: None,
            discovery_loading: false,
            discovery_complete: false,
            discovery_error: None,
            discovery_request: None,
            manual_shocker_id: String::new(),
            logs_open: false,
            save_error: None,
        }
    }

    fn start_discovery(&mut self, ctx: &egui::Context) {
        if self.changes.api_token.trim().is_empty() {
            self.discovery_error = Some("Enter an API token before discovering shockers.".into());
            return;
        }

        let (tx, rx) = mpsc::channel();
        let server = self.changes.api_server.clone();
        let token = self.changes.api_token.clone();
        let ctx = ctx.clone();
        self.discovery_request = Some((server.clone(), token.clone()));
        self.discovery_rx = Some(rx);
        self.discovery_loading = true;
        self.discovery_complete = false;
        self.discovery_error = None;
        tokio::spawn(async move {
            let result = openshock::discover_shockers(&server, &token).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    fn poll_discovery(&mut self) {
        let received = self.discovery_rx.as_ref().map(|rx| rx.try_recv());
        match received {
            Some(Ok(result)) => {
                self.discovery_loading = false;
                self.discovery_rx = None;
                let current = (
                    self.changes.api_server.clone(),
                    self.changes.api_token.clone(),
                );
                if self.discovery_request.as_ref() != Some(&current) {
                    self.discovery_error = Some(
                        "Connection settings changed. Discover shockers again to refresh the list."
                            .into(),
                    );
                    return;
                }
                match result {
                    Ok(shockers) => {
                        self.discovery_error = None;
                        self.discovered = shockers;
                        self.discovery_complete = true;
                        info!(target: "GUI", "Discovered {} shocker(s)", self.discovered.len());
                    }
                    Err(error) => {
                        self.discovery_error = Some(error);
                    }
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.discovery_loading = false;
                self.discovery_rx = None;
                self.discovery_error =
                    Some("Shockers could not be discovered. Please try again.".into());
            }
            _ => {}
        }
    }

    fn toggle_shocker(&mut self, id: &str, selected: bool) {
        if selected {
            if !self.changes.shocker_ids.iter().any(|item| item == id) {
                self.changes.shocker_ids.push(id.to_owned());
            }
        } else {
            self.changes.shocker_ids.retain(|item| item != id);
        }
    }

    fn save(&mut self) -> bool {
        debug!(target: "GUI", "Saving settings");
        if let Ok(mut owned_config) = self.config.try_write() {
            *owned_config = self.changes.clone();
            owned_config.write_to_file("config.json");
            true
        } else {
            false
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_discovery();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("CS2 Shock");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(self.logs_open, "Logs").clicked() {
                        self.logs_open = !self.logs_open;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Connection");
                ui.label("Use your OpenShock API token to find and select devices.");
                ui.add_space(6.0);
                ui.label("API token");
                ui.add(
                    egui::TextEdit::singleline(&mut self.changes.api_token)
                        .password(true)
                        .desired_width(f32::INFINITY),
                );
                ui.label("API server");
                ui.add(
                    egui::TextEdit::singleline(&mut self.changes.api_server)
                        .desired_width(f32::INFINITY),
                );
                if let Some((server, token)) = &self.discovery_request {
                    if server != &self.changes.api_server || token != &self.changes.api_token {
                        self.discovered.clear();
                        self.discovery_complete = false;
                        self.discovery_error = None;
                    }
                }

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.discovery_loading, Button::new("Discover shockers"))
                        .clicked()
                    {
                        self.start_discovery(ctx);
                    }
                    if self.discovery_loading {
                        ui.label("Searching...");
                    }
                });
                if let Some(error) = &self.discovery_error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                    ui.label("You can still add a shocker ID manually below.");
                }

                if !self.discovered.is_empty() {
                    ui.add_space(4.0);
                    ui.label("Available shockers");
                    for shocker in self.discovered.clone() {
                        let mut selected = self.changes.shocker_ids.contains(&shocker.id);
                        let label = if shocker.name.trim().is_empty() {
                            shocker.id.clone()
                        } else {
                            format!("{}  ({})", shocker.name, shocker.id)
                        };
                        if ui.checkbox(&mut selected, label).changed() {
                            self.toggle_shocker(&shocker.id, selected);
                        }
                    }
                } else if self.discovery_complete {
                    ui.weak("No shockers were found for this account.");
                }

                ui.add_space(4.0);
                ui.label("Selected shockers");
                if self.changes.shocker_ids.is_empty() {
                    ui.weak("No shockers selected. Add an ID or discover devices.");
                } else {
                    let selected = self.changes.shocker_ids.clone();
                    for id in selected {
                        ui.horizontal(|ui| {
                            ui.label(&id);
                            if ui.small_button("Remove").clicked() {
                                self.changes.shocker_ids.retain(|item| item != &id);
                            }
                        });
                    }
                }
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.manual_shocker_id)
                            .hint_text("Shocker ID"),
                    );
                    if ui.button("Add ID").clicked() {
                        let id = self.manual_shocker_id.trim().to_owned();
                        if !id.is_empty() && !self.changes.shocker_ids.contains(&id) {
                            self.changes.shocker_ids.push(id);
                            self.manual_shocker_id.clear();
                        }
                    }
                });

                ui.separator();
                ui.heading("Behavior");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.changes.shock_mode, ShockMode::Random, "Random");
                    ui.selectable_value(
                        &mut self.changes.shock_mode,
                        ShockMode::LastHitPercentage,
                        "Last hit %",
                    );
                });
                ui.horizontal(|ui| {
                    let label = ui.label("Intensity");
                    ui.add(
                        DragValue::new(&mut self.changes.min_intensity)
                            .speed(1)
                            .range(0..=self.changes.max_intensity)
                            .prefix("Min "),
                    )
                    .labelled_by(label.id);
                    ui.add(
                        DragValue::new(&mut self.changes.max_intensity)
                            .speed(1)
                            .range(self.changes.min_intensity..=100)
                            .prefix("Max "),
                    )
                    .labelled_by(label.id);
                });
                ui.horizontal(|ui| {
                    let label = ui.label("Duration");
                    ui.add(
                        DragValue::new(&mut self.changes.min_duration)
                            .speed(1)
                            .range(1..=self.changes.max_duration)
                            .prefix("Min "),
                    )
                    .labelled_by(label.id);
                    ui.add(
                        DragValue::new(&mut self.changes.max_duration)
                            .speed(1)
                            .range(self.changes.min_duration..=15)
                            .prefix("Max "),
                    )
                    .labelled_by(label.id);
                });
                ui.checkbox(&mut self.changes.beep_on_match_start, "Beep on match start");
                ui.checkbox(&mut self.changes.beep_on_round_start, "Beep on round start");

                ui.separator();
                ui.horizontal(|ui| {
                    let can_test_beep = !self.changes.api_token.trim().is_empty()
                        && !self.changes.api_server.trim().is_empty()
                        && self
                            .changes
                            .shocker_ids
                            .iter()
                            .any(|id| !id.trim().is_empty());
                    if ui
                        .add_enabled(can_test_beep, Button::new("Test beep"))
                        .on_hover_text(
                            "Sends using the unsaved token, server, and selected shockers above.",
                        )
                        .clicked()
                    {
                        info!(target: "GUI", "Sending test beep with current form settings");
                        let test_config = Arc::new(RwLock::new(self.changes.clone()));
                        tokio::spawn(async move {
                            openshock::beep(test_config, 1).await;
                        });
                    }
                    ui.weak("Uses current form values, including unsaved edits.");
                });
                ui.horizontal(|ui| {
                    let saved = self.config.try_read().map(|c| c.clone()).ok();
                    let changed = saved.as_ref() != Some(&self.changes);
                    if ui
                        .add_enabled(changed && saved.is_some(), Button::new("Reset"))
                        .clicked()
                    {
                        if let Some(saved) = saved {
                            self.changes = saved;
                        }
                    }
                    if ui.add_enabled(changed, Button::new("Save")).clicked() {
                        if self.changes.validate() {
                            self.save_error = if self.save() {
                                None
                            } else {
                                Some("Settings are busy. Please try saving again.".into())
                            };
                        } else {
                            self.save_error = Some(
                                "Check the intensity and duration ranges before saving.".into(),
                            );
                        }
                    }
                });
                if let Some(error) = &self.save_error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
        });

        if self.logs_open {
            egui::Window::new("Application logs")
                .open(&mut self.logs_open)
                .default_size([600.0, 320.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            crate::app_log::clear_logs();
                        }
                    });
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for line in crate::app_log::recent_logs() {
                                ui.monospace(line);
                            }
                        });
                });
        }

        crate::app_log::set_repaint_context(self.logs_open.then(|| ctx.clone()));

        if ctx.input(|i| i.viewport().close_requested()) {
            info!(target: "GUI", "Closing");
            process::exit(0);
        }
    }
}
