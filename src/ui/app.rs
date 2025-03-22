use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::{Application, Command, Element, Length, Theme, Subscription, time};
use std::time::Duration;

use crate::cleaning::clean_temp_files;
use crate::cleaning::CleaningSummary;
use crate::monitoring::system_info::{self, SystemStatus};
use crate::ui::widgets;

pub struct RustyCleanApp {
    cleaning_result: Option<Result<CleaningSummary, String>>,
    is_cleaning: bool,
    system_status: Option<SystemStatus>,
    monitoring_active: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    CleanTempFiles,
    CleaningCompleted(Result<CleaningSummary, String>),
    ToggleMonitoring,
    UpdateSystemStatus,
    SystemStatusUpdated(SystemStatus),
}

impl Application for RustyCleanApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        // Initialisiere das System-Monitoring
        system_info::init_monitoring();
        
        (
            Self {
                cleaning_result: None,
                is_cleaning: false,
                system_status: None,
                monitoring_active: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ZentifyCleaner - Windows Optimierungstool")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CleanTempFiles => {
                self.is_cleaning = true;
                self.cleaning_result = None;

                Command::perform(
                    async { clean_temp_files() },
                    Message::CleaningCompleted,
                )
            }
            Message::CleaningCompleted(result) => {
                self.cleaning_result = Some(result);
                self.is_cleaning = false;
                Command::none()
            }
            Message::ToggleMonitoring => {
                self.monitoring_active = !self.monitoring_active;
                
                if self.monitoring_active {
                    // Sofort die Systemdaten laden, wenn aktiviert
                    Command::perform(
                        async { 
                            match system_info::get_system_status() {
                                Ok(status) => status,
                                Err(_) => SystemStatus {
                                    cpu_usage: 0.0,
                                    memory_used: 0,
                                    memory_total: 0,
                                    disk_used: 0,
                                    disk_total: 0,
                                    top_processes: Vec::new(),
                                }
                            }
                        },
                        Message::SystemStatusUpdated,
                    )
                } else {
                    self.system_status = None;
                    Command::none()
                }
            }
            Message::UpdateSystemStatus => {
                if self.monitoring_active {
                    Command::perform(
                        async { 
                            match system_info::get_system_status() {
                                Ok(status) => status,
                                Err(_) => SystemStatus {
                                    cpu_usage: 0.0,
                                    memory_used: 0,
                                    memory_total: 0,
                                    disk_used: 0,
                                    disk_total: 0,
                                    top_processes: Vec::new(),
                                }
                            }
                        },
                        Message::SystemStatusUpdated,
                    )
                } else {
                    Command::none()
                }
            }
            Message::SystemStatusUpdated(status) => {
                self.system_status = Some(status);
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.monitoring_active {
            // Aktualisiere die Systemdaten alle 2 Sekunden
            time::every(Duration::from_secs(2)).map(|_| Message::UpdateSystemStatus)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        let header = widgets::header("RustyClean", "Windows Optimierungstool");

        // Haupt-Bereich mit Kacheln für verschiedene Funktionen
        let mut features = column![].spacing(20).width(Length::Fill);

        // Kachel für temporäre Dateien
        let temp_files_card = widgets::feature_card(
            "Temporäre Dateien",
            "Entfernt temporäre Dateien aus Windows- und Benutzerverzeichnissen",
            if self.is_cleaning { "Bereinigung läuft..." } else { "Jetzt bereinigen" },
            Message::CleanTempFiles,
            self.is_cleaning,
        );

        features = features.push(temp_files_card);

        // Bereinigungsergebnisse anzeigen, wenn vorhanden
        if let Some(result) = &self.cleaning_result {
            let result_content = match result {
                Ok(summary) => {
                    let mut content = column![
                        text(format!(
                            "Bereinigt: {} Dateien ({})",
                            summary.deleted_files,
                            summary.formatted_size()
                        ))
                        .size(18)
                    ]
                    .spacing(10);

                    if !summary.errors.is_empty() {
                        let mut errors_list = column![text("Fehler:").size(16)].spacing(5);

                        for error in &summary.errors[..std::cmp::min(summary.errors.len(), 5)] {
                            errors_list = errors_list.push(text(error).size(14));
                        }

                        if summary.errors.len() > 5 {
                            errors_list = errors_list.push(
                                text(format!("... und {} weitere", summary.errors.len() - 5))
                                    .size(14),
                            );
                        }

                        content = content.push(errors_list);
                    }

                    content
                }
                Err(error) => column![text(format!("Fehler: {}", error)).size(18)],
            };

            let result_card = container(result_content)
                .style(iced::theme::Container::Box)
                .width(Length::Fill)
                .padding(20);

            features = features.push(result_card);
        }

        // Systemüberwachung-Kachel
        let monitoring_card = widgets::feature_card(
            "Systemüberwachung",
            "Überwacht CPU, Speicher und Festplattennutzung in Echtzeit",
            if self.monitoring_active {
                "Überwachung stoppen"
            } else {
                "Überwachung starten"
            },
            Message::ToggleMonitoring,
            false,
        );

        features = features.push(monitoring_card);

        // Systemstatus anzeigen, wenn die Überwachung aktiv ist
        if self.monitoring_active {
            let status_content = if let Some(status) = &self.system_status {
                // CPU-Auslastung
                let cpu_usage = column![
                    text("CPU-Auslastung").size(18),
                    text(system_info::format_percentage(status.cpu_usage)).size(24)
                ]
                .spacing(5)
                .padding(10)
                .width(Length::Fill)
                .align_items(iced::Alignment::Center);

                // Speichernutzung
                let mem_percentage = (status.memory_used as f64 / status.memory_total as f64 * 100.0) as f32;
                let memory_usage = column![
                    text("Speichernutzung").size(18),
                    text(format!(
                        "{} / {} ({})",
                        system_info::format_bytes(status.memory_used),
                        system_info::format_bytes(status.memory_total),
                        system_info::format_percentage(mem_percentage)
                    ))
                    .size(16)
                ]
                .spacing(5)
                .padding(10)
                .width(Length::Fill)
                .align_items(iced::Alignment::Center);

                // Festplattennutzung
                let disk_percentage = if status.disk_total > 0 {
                    (status.disk_used as f64 / status.disk_total as f64 * 100.0) as f32
                } else {
                    0.0
                };
                let disk_usage = column![
                    text("Festplattennutzung").size(18),
                    text(format!(
                        "{} / {} ({})",
                        system_info::format_bytes(status.disk_used),
                        system_info::format_bytes(status.disk_total),
                        system_info::format_percentage(disk_percentage)
                    ))
                    .size(16)
                ]
                .spacing(5)
                .padding(10)
                .width(Length::Fill)
                .align_items(iced::Alignment::Center);

                // Hauptlayout der Systemstatistiken
                let system_metrics = row![cpu_usage, memory_usage, disk_usage]
                    .spacing(20)
                    .padding(10)
                    .width(Length::Fill);

                // Top-Prozesse
                let mut process_list = column![text("Top-Prozesse nach CPU-Nutzung:").size(18)]
                    .spacing(10)
                    .padding(10);

                for proc in &status.top_processes {
                    process_list = process_list.push(
                        row![
                            text(&proc.name).size(14).width(Length::FillPortion(6)),
                            text(system_info::format_percentage(proc.cpu_usage))
                                .size(14)
                                .width(Length::FillPortion(2)),
                            text(system_info::format_bytes(proc.memory_usage))
                                .size(14)
                                .width(Length::FillPortion(3)),
                        ]
                        .spacing(10)
                        .padding(5)
                    );
                }

                column![system_metrics, process_list]
            } else {
                column![text("Lade Systemdaten...").size(18)]
                    .padding(20)
                    .align_items(iced::Alignment::Center)
            };

            let status_card = container(status_content)
                .style(iced::theme::Container::Box)
                .width(Length::Fill)
                .padding(10);

            features = features.push(status_card);
        }

        // Platzhalter für weitere Funktionen (deaktiviert)
        let coming_soon_features = ["Registry-Cleanup", "Autostart-Manager"];

        for feature in coming_soon_features.iter() {
            let feature_card = widgets::feature_card(
                feature,
                "Demnächst verfügbar",
                "In Entwicklung",
                Message::CleanTempFiles, // Wird nie ausgeführt (deaktiviert)
                true, // Immer deaktiviert
            );

            features = features.push(feature_card);
        }

        // Footer
        let footer = row![text("© 2025 fSN").size(14)]
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .align_items(iced::Alignment::Center);

        let content = column![
            header,
            scrollable(features).height(Length::Fill),
            footer
        ]
        .spacing(30)
        .padding(20)
        .align_items(iced::Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}
