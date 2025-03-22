use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, Pid, PidExt};
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
    if let Ok(mut sys) = SYSTEM.lock() {
        sys.refresh_all();
    }
}

/// Aktualisiert die Systemdaten und gibt einen Snapshot zurück
pub fn get_system_status() -> Result<SystemStatus, &'static str> {
    let mut sys_guard = match SYSTEM.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("Konnte keine Sperre auf das System-Objekt erhalten"),
    };
    
    sys_guard.refresh_all();

    // CPU-Auslastung berechnen (Durchschnitt aller Kerne)
    let cpu_usage = if sys_guard.cpus().is_empty() {
        0.0
    } else {
        let total: f32 = sys_guard.cpus().iter().map(|p| p.cpu_usage()).sum();
        total / sys_guard.cpus().len() as f32
    };

    // Arbeitsspeicher-Informationen direkt in KB verwenden
    let memory_used = sys_guard.used_memory();  // Direkt in KB
    let memory_total = sys_guard.total_memory();  // Direkt in KB

    // Nimm nur das Hauptlaufwerk (normalerweise C:)
    let mut disk_used = 0;
    let mut disk_total = 0;
    
    // Finde das Hauptsystemlaufwerk (i.d.R. das mit der größten Kapazität und echtem Dateisystem)
    for disk in sys_guard.disks() {
        let mount_point = disk.mount_point().to_string_lossy();
        // Auf Windows ist das Systemlaufwerk typischerweise C:
        if (mount_point.contains("C:") || mount_point.contains("/")) && disk.total_space() > 10_000_000_000 {
            disk_used = disk.total_space() - disk.available_space();
            disk_total = disk.total_space();
            break; // Stoppen nach dem ersten passenden Laufwerk
        }
    }
    
    // Fallback: Nimm das größte Laufwerk, falls kein C: gefunden wurde
    if disk_total == 0 {
        let mut max_size = 0;
        for disk in sys_guard.disks() {
            if disk.total_space() > max_size && disk.total_space() < 10_000_000_000_000 {
                max_size = disk.total_space();
                disk_used = disk.total_space() - disk.available_space();
                disk_total = disk.total_space();
            }
        }
    }

    // Alle Prozesse sammeln
    let mut processes: Vec<ProcessInfo> = Vec::new();
    let cpu_count = sys_guard.cpus().len() as f32;
    
    for (pid, process) in sys_guard.processes() {
        processes.push(ProcessInfo {
            name: process.name().to_string(),
            pid: pid.as_u32(),
            cpu_usage: process.cpu_usage() / cpu_count, // CPU-Nutzung durch Anzahl der Kerne teilen
            memory_usage: process.memory(), // Speichernutzung direkt in KB ohne falsche Umrechnung
        });
    }

    // Prozesse nach CPU-Auslastung sortieren (absteigend)
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

    // Top-5 Prozesse auswählen
    let top_processes = processes.into_iter().take(5).collect();

    Ok(SystemStatus {
        cpu_usage,
        memory_used,
        memory_total,
        disk_used,
        disk_total,
        top_processes,
    })
}

/// Formatiert Bytes in eine lesbare Größe
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    // Spezieller Fall für RAM (typischerweise in KB)
    if bytes > 0 && bytes < 1024 * 1024 * 1024 {
        size /= 1024.0;
        unit_idx = 1; // KB
    }

    // Standard-Konvertierung für alle anderen Fälle (Bytes)
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Formatiert den Prozentsatz
pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value)
}
