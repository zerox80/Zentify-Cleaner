use std::fs;
use std::path::{Path, PathBuf};
use std::env;

/// Löscht temporäre Dateien aus standard Verzeichnissen und weiteren Orten
pub fn clean_temp_files() -> Result<CleaningSummary, String> {
    let mut summary = CleaningSummary {
        deleted_files: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    // Erweiterte Liste von temporären Verzeichnissen
    let temp_directories = vec![
        get_windows_temp_dir(),                // Windows\Temp
        get_user_temp_dir(),                   // Benutzer-TEMP
        get_prefetch_dir(),                    // Windows\Prefetch (Vorfetch-Dateien)
        get_windows_update_dir(),              // SoftwareDistribution\Download
        get_recycle_bin(),                     // Papierkorb
        get_ie_cache_dir(),                    // Temporary Internet Files
        get_browser_cache_dir("Chrome"),       // Chrome Cache
        get_browser_cache_dir("Edge"),         // Edge Cache
        get_browser_cache_dir("Firefox"),      // Firefox Cache
        get_recent_docs_dir(),                 // Recent-Dokumente
        get_thumbnails_cache_dir(),            // Windows-Miniaturansichten
    ];

    // Leere die Verzeichnisse
    for dir in temp_directories {
        if let Some(dir_path) = dir {
            if let Err(e) = clean_directory(&dir_path, &mut summary) {
                summary.errors.push(format!("Verzeichnis {}: {}", dir_path.display(), e));
            }
        }
    }

    // Spezielle Dateitypen bereinigen
    if let Some(windows_dir) = get_windows_dir() {
        if let Err(e) = clean_file_type(&windows_dir, "*.log", &mut summary) {
            summary.errors.push(format!("Log-Dateien Bereinigung: {}", e));
        }
        if let Err(e) = clean_file_type(&windows_dir, "*.dmp", &mut summary) {
            summary.errors.push(format!("Dump-Dateien Bereinigung: {}", e));
        }
        if let Err(e) = clean_file_type(&windows_dir, "*.tmp", &mut summary) {
            summary.errors.push(format!("TMP-Dateien Bereinigung: {}", e));
        }
    }

    Ok(summary)
}

/// Rekursives Löschen temporärer Dateien in einem Verzeichnis
fn clean_directory(dir: &Path, summary: &mut CleaningSummary) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Err("Existiert nicht oder ist kein Verzeichnis".into());
    }

    // Versuche, alle Einträge zu lesen
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            // Bei Zugriffsproblemen nicht abbrechen, sondern als Fehler erfassen und weitermachen
            summary.errors.push(format!("Verzeichnis {} konnte nicht gelesen werden: {}", dir.display(), e));
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                summary.errors.push(format!("Ungültiger Eintrag: {}", e));
                continue;
            }
        };
        
        let path = entry.path();
        
        // Überspringe geschützte Systemdateien
        if is_protected_file(&path) {
            continue;
        }
        
        if path.is_dir() {
            // Rekursive Bereinigung des Unterverzeichnisses
            let _ = clean_directory(&path, summary);
            
            // Lösche leeres Verzeichnis nach Bereinigung
            if let Err(e) = fs::remove_dir(&path) {
                // Ignoriere Fehler beim Löschen nicht-leerer Verzeichnisse
                if !e.to_string().contains("Das Verzeichnis ist nicht leer") {
                    summary.errors.push(format!("Konnte Verzeichnis {} nicht löschen: {}", path.display(), e));
                }
            }
        } else {
            // Datei löschen und Größe erfassen
            if let Ok(metadata) = fs::metadata(&path) {
                match fs::remove_file(&path) {
                    Ok(()) => {
                        summary.deleted_files += 1;
                        summary.total_size += metadata.len();
                    },
                    Err(e) => {
                        // Ignoriere Dateien, die gerade verwendet werden
                        if !e.to_string().contains("Zugriff verweigert") && 
                           !e.to_string().contains("verwendet") {
                            summary.errors.push(format!("Konnte {} nicht löschen: {}", path.display(), e));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Prüft, ob eine Datei geschützt werden sollte
fn is_protected_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    
    // Kritische Systemdateien und -verzeichnisse schützen
    path_str.contains("system32") || 
    path_str.contains("systemapps") || 
    path_str.contains("winre") || 
    path_str.contains("bootmgr") || 
    path_str.ends_with(".sys") ||
    path_str.ends_with(".dll") ||
    path_str.ends_with(".exe")
}

/// Bereinigt spezifische Dateitypen (z.B. *.log) im angegebenen Verzeichnis und Unterverzeichnissen
fn clean_file_type(dir: &Path, pattern: &str, summary: &mut CleaningSummary) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Err("Existiert nicht oder ist kein Verzeichnis".into());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Verzeichnis konnte nicht gelesen werden: {}", e)),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                summary.errors.push(format!("Ungültiger Eintrag: {}", e));
                continue;
            }
        };
        
        let path = entry.path();
        
        if path.is_dir() {
            // Rekursive Suche in Unterverzeichnissen
            let _ = clean_file_type(&path, pattern, summary);
        } else if matches_pattern(&path, pattern) && !is_protected_file(&path) {
            // Passende Datei gefunden - versuche zu löschen
            if let Ok(metadata) = fs::metadata(&path) {
                match fs::remove_file(&path) {
                    Ok(()) => {
                        summary.deleted_files += 1;
                        summary.total_size += metadata.len();
                    },
                    Err(e) => {
                        if !e.to_string().contains("Zugriff verweigert") {
                            summary.errors.push(format!("Konnte {} nicht löschen: {}", path.display(), e));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Prüft, ob ein Dateipfad einem Muster (*-Wildcard unterstützt) entspricht
fn matches_pattern(path: &Path, pattern: &str) -> bool {
    if let Some(file_name) = path.file_name() {
        let file_name_str = file_name.to_string_lossy().to_lowercase();
        let pattern = pattern.to_lowercase();
        
        if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            return file_name_str.ends_with(suffix);
        } else if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len()-1];
            return file_name_str.starts_with(prefix);
        } else if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            return file_name_str.starts_with(parts[0]) && file_name_str.ends_with(parts[1]);
        }
        
        // Exakte Übereinstimmung
        return file_name_str == pattern;
    }
    false
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

/// Windows-Verzeichnis
fn get_windows_dir() -> Option<PathBuf> {
    env::var("SystemRoot").ok().map(PathBuf::from)
}

/// Windows Prefetch-Verzeichnis
fn get_prefetch_dir() -> Option<PathBuf> {
    env::var("SystemRoot")
        .ok()
        .map(|root| PathBuf::from(root).join("Prefetch"))
}

/// Windows Update Download-Verzeichnis
fn get_windows_update_dir() -> Option<PathBuf> {
    env::var("SystemRoot")
        .ok()
        .map(|root| PathBuf::from(root).join("SoftwareDistribution").join("Download"))
}

/// Papierkorb
fn get_recycle_bin() -> Option<PathBuf> {
    // Der Papierkorb ist pro Laufwerk unter $Recycle.Bin versteckt
    let drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    Some(PathBuf::from(format!("{}\\$Recycle.Bin", drive)))
}

/// Internet Explorer Cache
fn get_ie_cache_dir() -> Option<PathBuf> {
    env::var("LOCALAPPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("INetCache"))
}

/// Browser-Cache-Verzeichnisse
fn get_browser_cache_dir(browser: &str) -> Option<PathBuf> {
    let local_app_data = env::var("LOCALAPPDATA").ok()?;
    
    match browser {
        "Chrome" => Some(PathBuf::from(&local_app_data)
            .join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default")
            .join("Cache")),
        "Edge" => Some(PathBuf::from(&local_app_data)
            .join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default")
            .join("Cache")),
        "Firefox" => {
            // Firefox speichert Profildaten an einem anderen Ort
            let app_data = env::var("APPDATA").ok()?;
            Some(PathBuf::from(app_data)
                .join("Mozilla")
                .join("Firefox")
                .join("Profiles"))
        },
        _ => None,
    }
}

/// Windows Recent-Dokumente
fn get_recent_docs_dir() -> Option<PathBuf> {
    env::var("APPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata).join("Microsoft").join("Windows").join("Recent"))
}

/// Windows-Miniaturansichten-Cache
fn get_thumbnails_cache_dir() -> Option<PathBuf> {
    env::var("LOCALAPPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata).join("Microsoft").join("Windows").join("Explorer").join("thumbcache_*.db"))
}

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