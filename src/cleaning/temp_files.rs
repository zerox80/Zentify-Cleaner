use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Ergebniszusammenfassung
#[derive(Debug, Clone)]
pub struct CleaningSummary {
    pub deleted_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
}

impl CleaningSummary {
    /// Menschlesbare Dateigröße
    pub fn formatted_size(&self) -> String {
        const UNITS: [&str; 4] = ["Bytes", "KB", "MB", "GB"];
        let mut size = self.total_size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < 3 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Löscht temporäre Dateien aus standard Verzeichnissen
pub fn clean_temp_files() -> Result<CleaningSummary, String> {
    let mut summary = CleaningSummary {
        deleted_files: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    let temp_directories = vec![
        get_windows_temp_dir(),
        get_user_temp_dir(),
    ];

    for dir in temp_directories {
        if let Some(dir_path) = dir {
            if let Err(e) = clean_directory(&dir_path, &mut summary) {
                summary.errors.push(format!("Verzeichnis {}: {}", dir_path.display(), e));
            }
        }
    }

    Ok(summary)
}

/// Rekursives Löschen temporärer Dateien in einem Verzeichnis
fn clean_directory(dir: &Path, summary: &mut CleaningSummary) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Err("Existiert nicht oder ist kein Verzeichnis".into());
    }

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Verzeichnis konnte nicht gelesen werden: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Ungültiger Eintrag: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Err(e) = clean_directory(&path, summary) {
                summary.errors.push(format!("{}: {}", path.display(), e));
            }
            // Lösche leeres Verzeichnis nach Bereinigung
            let _ = fs::remove_dir(&path);
        } else {
            let metadata = fs::metadata(&path).ok();
            match fs::remove_file(&path) {
                Ok(()) => {
                    summary.deleted_files += 1;
                    if let Some(m) = metadata {
                        summary.total_size += m.len();
                    }
                },
                Err(e) => summary.errors.push(
                    format!("Konnte {} nicht löschen: {}", path.display(), e)
                )
            }
        }
    }

    Ok(())
}

/// Windows System-TEMP mit Umgebungsvariable
fn get_windows_temp_dir() -> Option<PathBuf> {
    env::var("SystemRoot")
        .ok()
        .map(|root| PathBuf::from(root).join("Temp"))
}

/// Benutzer-TEMP mit Umgebungsvariable
fn get_user_temp_dir() -> Option<PathBuf> {
    env::var("TEMP").ok().map(PathBuf::from)
}
