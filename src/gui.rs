use eframe::{
    egui::{CentralPanel, ComboBox, ViewportBuilder},
    run_native, App, NativeOptions,
};

use crate::{get_devices, Device};

pub fn run_gui() {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320., 240.]),
        ..Default::default()
    };
    run_native(
        concat!("OwOtility v", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|_| Box::<Gui>::default()),
    )
    .unwrap();
}

struct Gui {
    dev: Device,
    available: Vec<String>,
    selected: String,
}

impl Default for Gui {
    fn default() -> Self {
        let available = get_devices();
        Self {
            dev: Device::new(&available[0]),
            selected: available[0].clone(),
            available,
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("OwOtility");
                let mut selected = usize::MAX;
                ui.add_space(16.);
                ComboBox::from_label("Select device")
                    .selected_text(&self.selected)
                    .show_ui(ui, |ui| {
                        for i in 0..self.available.len() {
                            ui.selectable_value(&mut selected, i, &self.available[i]);
                        }
                    });
                if selected != usize::MAX {
                    self.selected = self.available[selected].clone();
                    self.dev = Device::new(&self.available[selected]);
                }
            });
        });
    }
}
