//! Theme management for the application.

use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub visuals: egui::Visuals,
}

pub fn dark_theme() -> Theme {
    Theme {
        name: "Dark",
        visuals: egui::Visuals::dark(),
    }
}

pub fn light_theme() -> Theme {
    Theme {
        name: "Light",
        visuals: egui::Visuals::light(),
    }
}

pub fn initial_theme() -> Theme {
    dark_theme()
} 