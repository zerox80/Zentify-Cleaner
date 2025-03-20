use rustyclean::ui::RustyCleanApp;
use iced::{Application, Settings};

fn main() -> iced::Result {
    // Starte die GUI-Anwendung
    RustyCleanApp::run(Settings::default())
}
