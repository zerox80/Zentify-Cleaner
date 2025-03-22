use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;

/// Ergebniszusammenfassung
#[derive(Debug, Clone)]
pub struct CleaningSummary {
    pub deleted_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
    pub skipped_files: usize,
    pub cleaned_locations: HashMap<String, LocationSummary>,
}

/// Zusammenfassung für einen bestimmten Ort
#[derive(Debug, Clone)]
pub struct LocationSummary {
    pub location_name: String,
    pub deleted_files: usize,
    pub total_size: u64,
}

impl CleaningSummary {
    /// Erstellt eine neue leere Zusammenfassung
    pub fn new() -> Self {
        Self {
            deleted_files: 0,
            total_size: 0,
            errors: Vec::new(),
            skipped_files: 0,
            cleaned_locations: HashMap::new(),
        }
    }

    /// Fügt Daten für einen bestimmten Bereinigungsort hinzu
    pub fn add_location_data(&mut self, location_name: &str, files: usize, size: u64) {
        self.cleaned_locations.insert(
            location_name.to_string(),
            LocationSummary {
                location_name: location_name.to_string(),
                deleted_files: files,
                total_size: size,
            },
        );
    }

    /// Menschlesbare Dateigröße
    pub fn formatted_size(&self) -> String {
        format_bytes(self.total_size)
    }
}

/// Optionen für die Bereinigung
#[derive(Debug, Clone)]
pub struct CleaningOptions {
    /// Minimales Alter der zu löschenden Dateien in Tagen
    pub min_file_age_days: u64,
    /// Ob Unterverzeichnisse bereinigt werden sollen
    pub recursive: bool,
    /// Ob leere Verzeichnisse gelöscht werden sollen
    pub remove_empty_dirs: bool,
    /// Ob bestimmte Dateiendungen gezielt gelöscht werden sollen
    pub target_extensions: Option<Vec<String>>,
    /// Maximale Anzahl an Dateien, die gelöscht werden sollen (0 = unbegrenzt)
    pub max_files: usize,
}

impl Default for CleaningOptions {
    fn default() -> Self {
        Self {
            min_file_age_days: 1,
            recursive: true,
            remove_empty_dirs: true,
            target_extensions: None,
            max_files: 0,
        }
    }
}

/// Löscht temporäre Dateien aus standard Verzeichnissen
pub fn clean_temp_files() -> Result<CleaningSummary, String> {
    clean_temp_files_with_options(CleaningOptions::default())
}

/// Löscht temporäre Dateien mit benutzerdefinierten Optionen
pub fn clean_temp_files_with_options(options: CleaningOptions) -> Result<CleaningSummary, String> {
    let mut summary = CleaningSummary::new();

    // Struktur: (Verzeichnispfad-Funktion, Standortname)
    let temp_locations: Vec<(fn() -> Option<PathBuf>, &str)> = vec![
        (get_windows_temp_dir, "Windows Temp"),
        (get_user_temp_dir, "Benutzer Temp"),
        (get_prefetch_dir, "Windows Prefetch"),
        (get_ie_cache_dir, "Internet Explorer Cache"),
        (get_chrome_cache_dir, "Chrome Cache"),
        (get_firefox_cache_dir, "Firefox Cache"),
        (get_edge_cache_dir, "Edge Cache"),
        (get_brave_cache_dir, "Brave Cache"),
        (get_thumbnails_cache_dir, "Windows Miniaturansichten"),
        (get_windows_update_dir, "Windows Update Cache"),
        // Papierkorb wurde entfernt
    ];

    for (dir_fn, location_name) in temp_locations {
        if let Some(dir) = dir_fn() {
            let location_files_before = summary.deleted_files;
            let location_size_before = summary.total_size;
            
            if let Err(e) = clean_directory(&dir, location_name, &mut summary, &options) {
                summary.errors.push(format!("Verzeichnis {} ({}): {}", dir.display(), location_name, e));
            }
            
            // Statistiken für diesen Standort speichern
            let files_cleaned = summary.deleted_files - location_files_before;
            let size_cleaned = summary.total_size - location_size_before;
            
            if files_cleaned > 0 {
                summary.add_location_data(location_name, files_cleaned, size_cleaned);
            }
        }
    }

    Ok(summary)
}

/// Rekursives Löschen temporärer Dateien in einem Verzeichnis
fn clean_directory(
    dir: &Path,
    location_name: &str,
    summary: &mut CleaningSummary,
    options: &CleaningOptions
) -> Result<(), String> {
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("{} existiert nicht oder ist kein Verzeichnis", dir.display()));
    }

    // Prüfen, ob maximale Dateianzahl bereits erreicht wurde
    if options.max_files > 0 && summary.deleted_files >= options.max_files {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Verzeichnis {} konnte nicht gelesen werden: {}", dir.display(), e))
    };

    // Zuerst alle Einträge sammeln, damit wir nicht während der Iteration Verzeichnisse löschen
    let mut file_entries = Vec::new();
    let mut dir_entries = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                summary.errors.push(format!("Ungültiger Eintrag: {}", e));
                continue;
            }
        };
        
        let path = entry.path();
        
        if path.is_dir() {
            dir_entries.push(path);
        } else {
            file_entries.push(path);
        }
    }

    // Dateien verarbeiten
    for path in file_entries {
        // Überspringe, wenn maximale Dateianzahl erreicht
        if options.max_files > 0 && summary.deleted_files >= options.max_files {
            break;
        }

        // Überspringe spezielle Systemdateien
        if should_skip_file(&path, location_name) {
            summary.skipped_files += 1;
            continue;
        }

        // Überprüfe Dateiendung, falls gewünscht
        if let Some(extensions) = &options.target_extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    summary.skipped_files += 1;
                    continue;
                }
            } else {
                summary.skipped_files += 1;
                continue;
            }
        }

        // Nur Dateien löschen, die älter als die angegebene Zeit sind
        if is_file_older_than_days(&path, options.min_file_age_days).unwrap_or(false) {
            let metadata = fs::metadata(&path).ok();
            let file_size = metadata.as_ref().map_or(0, |m| m.len());
            
            match fs::remove_file(&path) {
                Ok(()) => {
                    summary.deleted_files += 1;
                    summary.total_size += file_size;
                },
                Err(e) => {
                    // Ignoriere "in Benutzung" Fehler
                    if !is_file_in_use_error(&e) {
                        summary.errors.push(format!("Konnte {} nicht löschen: {}", path.display(), e));
                    }
                    summary.skipped_files += 1;
                }
            }
        } else {
            summary.skipped_files += 1;
        }
    }

    // Unterverzeichnisse rekursiv verarbeiten, wenn aktiviert
    if options.recursive {
        for path in &dir_entries {
            let is_symlink = fs::symlink_metadata(path)
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false);
            if !is_symlink {
                clean_directory(path, location_name, summary, options)?;
            } else {
                summary.skipped_files += 1; // Optional: Symlink als übersprungen zählen
            }
        }
    }

    // Leere Verzeichnisse entfernen, wenn aktiviert
    if options.remove_empty_dirs {
        for path in &dir_entries {
            if is_directory_empty(path).unwrap_or(false) {
                if let Err(e) = fs::remove_dir(path) {
                    summary.errors.push(format!("Konnte leeres Verzeichnis {} nicht löschen: {}", path.display(), e));
                }
            }
        }
    }

    Ok(())
}

/// Überprüft, ob eine Datei übersprungen werden sollte
fn should_skip_file(path: &Path, location_name: &str) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name.starts_with(".") {
        return true;
    }
    
    let protected_files: &[&str] = match location_name {
        "Windows Temp" => &["desktop.ini", "thumbs.db"],
        "Benutzer Temp" => &["desktop.ini", "some_app_config"],
        "Windows Prefetch" => &[],
        "Internet Explorer Cache" => &["index.dat"],
        "Chrome Cache" => &[],
        "Firefox Cache" => &["places.sqlite", "cookies.sqlite"],
        "Edge Cache" => &[],
        "Brave Cache" => &[],
        "Windows Miniaturansichten" => &["thumbcache_*.db"],
        "Windows Update Cache" => &[],
        _ => &["desktop.ini", "thumbs.db", "ntuser.dat", "iconcache.db", "bootsect.bak", "pagefile.sys"],
    };
    
    protected_files.iter().any(|f| file_name.eq_ignore_ascii_case(f))
}

/// Prüft, ob ein Verzeichnis leer ist
fn is_directory_empty(dir: &Path) -> io::Result<bool> {
    Ok(fs::read_dir(dir)?.next().is_none())
}

/// Prüft, ob eine Datei älter als eine bestimmte Anzahl Tage ist
fn is_file_older_than_days(path: &Path, days: u64) -> io::Result<bool> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let now = SystemTime::now();
    
    match now.duration_since(modified) {
        Ok(duration) => Ok(duration >= Duration::from_secs(days * 24 * 60 * 60)),
        Err(_) => Ok(false), // Falls die Uhr zurückgesetzt wurde
    }
}

/// Prüft, ob ein Fehler darauf hindeutet, dass die Datei in Benutzung ist
#[cfg(windows)]
fn is_file_in_use_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::PermissionDenied ||
    error.raw_os_error() == Some(32) || // ERROR_SHARING_VIOLATION
    error.raw_os_error() == Some(33)    // ERROR_LOCK_VIOLATION
}

#[cfg(not(windows))]
fn is_file_in_use_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::PermissionDenied
}

/// Formatiert Bytes in eine lesbare Größe
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["Bytes", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_idx])
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

/// Windows Prefetch-Verzeichnis
fn get_prefetch_dir() -> Option<PathBuf> {
    env::var("SystemRoot")
        .ok()
        .map(|root| PathBuf::from(root).join("Prefetch"))
}

/// Internet Explorer Cache
fn get_ie_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("Microsoft\\Windows\\Temporary Internet Files"))
    } else {
        None
    }
}

/// Google Chrome Cache
fn get_chrome_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("Google\\Chrome\\User Data\\Default\\Cache"))
    } else {
        None
    }
}

/// Firefox Cache
fn get_firefox_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("Mozilla\\Firefox\\Profiles"))
    } else {
        None
    }
}

/// Microsoft Edge Cache
fn get_edge_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("Microsoft\\Edge\\User Data\\Default\\Cache"))
    } else {
        None
    }
}

/// Brave Browser Cache
fn get_brave_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache"))
    } else {
        None
    }
}

/// Windows Miniaturansichten Cache
fn get_thumbnails_cache_dir() -> Option<PathBuf> {
    if let Some(local_app_data) = env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
        Some(local_app_data.join("Microsoft\\Windows\\Explorer"))
    } else {
        None
    }
}

/// Windows Update-Cache
fn get_windows_update_dir() -> Option<PathBuf> {
    env::var("SystemRoot")
        .ok()
        .map(|root| PathBuf::from(root).join("SoftwareDistribution\\Download"))
}