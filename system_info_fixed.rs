use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt};
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Enthält alle gesammelten Systeminformationen
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: f32,             // CPU-Auslastung in Prozent
    pub memory_used: u64,           // Belegter Arbeitsspeicher in Bytes
    pub memory_total: u64,          // Gesamter Arbeitsspeicher in Bytes
    pub disk_used: u64,             // Belegter Festplattenplatz in Bytes
    pub disk_total: u64,            // Gesamter Festplattenplatz in Bytes
    pub top_processes: Vec<ProcessInfo>, // Top-Prozesse nach CPU-Nutzung
}

/// Informationen über einen einzelnen Prozess
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,      // Name des Prozesses
    pub pid: u32,          // Prozess-ID
    pub cpu_usage: f32,    // CPU-Auslastung des Prozesses
    pub memory_usage: u64, // Speichernutzung des Prozesses in Bytes
}

/// Singleton-System-Instanz (thread-sicher über Mutex)
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| {
    let mut sys = System::new_all();
    sys.refresh_all();
    Mutex::new(sys)
});

/// Initialisiert das System-Monitoring (aktualisiert einfach die Daten)
pub fn init_monitoring() {
    let mut sys = SYSTEM.lock().unwrap();
    sys.refresh_all();
}

/// Aktualisiert die Systemdaten und gibt einen Snapshot zurück
pub fn get_system_status() -> SystemStatus {
    let mut sys = SYSTEM.lock().unwrap();
    sys.refresh_all();

    // CPU-Auslastung berechnen (Durchschnitt aller Kerne)
    let cpu_usage = if sys.processors().is_empty() {
        0.0
    } else {
        let total: f32 = sys.processors().iter().map(|p| p.cpu_usage()).sum();
        total / sys.processors().len() as f32
    };

    // Arbeitsspeicher-Informationen (Bytes)
    let memory_used = sys.used_memory() * 1024;  // Umrechnung zu Bytes
    let memory_total = sys.total_memory() * 1024;  // Umrechnung zu Bytes

    // Festplatten-Informationen (nur erste Festplatte)
    let (disk_used, disk_total) = if !sys.disks().is_empty() {
        let disk = &sys.disks()[0];
        // Berechnung: belegter Speicher = Gesamt - verfügbarer Speicher
        (disk.total_space() - disk.available_space(), disk.total_space())
    } else {
        (0, 0)
    };

    // Alle Prozesse sammeln
    let mut processes: Vec<ProcessInfo> = sys.processes().iter().map(|(&pid, process)| {
        ProcessInfo {
            name: process.name().to_string(),
            pid: pid as u32,
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory() * 1024, // Umrechnung zu Bytes
        }
    }).collect();

    // Prozesse nach CPU-Auslastung sortieren (absteigend)
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

    // Top-5 Prozesse auswählen
    let top_processes = processes.into_iter().take(5).collect();

    SystemStatus {
        cpu_usage,
        memory_used,
        memory_total,
        disk_used,
        disk_total,
        top_processes,
    }
}

/// Formatiert Bytes in eine lesbare Größe
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < 3 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Formatiert den Prozentsatz
pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value)
}
