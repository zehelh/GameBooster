use eframe::egui;

pub fn centered_button(
    ui: &mut egui::Ui,
    text: &str,
    button_width: f32,
) -> egui::Response {
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_width() - button_width) / 2.0);
        ui.add(egui::Button::new(text).min_size(egui::vec2(button_width, 0.0)))
    })
    .inner
} 