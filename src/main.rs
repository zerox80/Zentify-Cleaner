use rustyclean::ui::RustyCleanApp;
use iced::{Application, Settings};

fn main() -> iced::Result {
    // Starte die GUI-Anwendung mit deaktiviertem Drag-and-Drop
    let mut settings = Settings::default();
    settings.window.platform_specific.drag_and_drop = false;
    
    RustyCleanApp::run(settings)
}
