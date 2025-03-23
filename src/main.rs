use rustyclean::ui::RustyCleanApp;
use iced::{Application, Settings};

#[cfg(windows)]
fn init_windows_com() {
    // Initialize COM in multi-threaded mode before anything else
    // to prevent OleInitialize conflicts
    unsafe {
        windows::Win32::System::Com::CoInitializeEx(
            std::ptr::null_mut(),
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        ).ok();
    }
}

fn main() -> iced::Result {
    // Initialize COM properly on Windows
    #[cfg(windows)]
    init_windows_com();
    
    // Starte die GUI-Anwendung
    RustyCleanApp::run(Settings::default())
}
