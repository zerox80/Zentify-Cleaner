use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use std::ffi::OsStr;

#[cfg(windows)]
use windows::Win32::Foundation::{ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION};

/// Verbessertes Error-Handling für Bereinigungsoperationen
#[derive(Debug)]
pub enum CleaningError {
    DirectoryNotFound(PathBuf),
    PermissionDenied(PathBuf),
    FileInUse(PathBuf),
    IoError(PathBuf, String),
    InvalidPath(String),
}

impl Clone for CleaningError {
    fn clone(&self) -> Self {
        match self {
            CleaningError::DirectoryNotFound(path) => CleaningError::DirectoryNotFound(path.clone()),
            CleaningError::PermissionDenied(path) => CleaningError::PermissionDenied(path.clone()),
            CleaningError::FileInUse(path) => CleaningError::FileInUse(path.clone()),
            CleaningError::IoError(path, err) => CleaningError::IoError(path.clone(), err.clone()),
            CleaningError::InvalidPath(msg) => CleaningError::InvalidPath(msg.clone()),
        }
    }
}

impl std::fmt::Display for CleaningError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CleaningError::DirectoryNotFound(path) => 
                write!(f, "Verzeichnis nicht gefunden: {}", path.display()),
            CleaningError::PermissionDenied(path) => 
                write!(f, "Keine Berechtigung: {}", path.display()),
            CleaningError::FileInUse(path) => 
                write!(f, "Datei in Verwendung: {}", path.display()),
            CleaningError::IoError(path, err) => 
                write!(f, "IO-Fehler bei {}: {}", path.display(), err),
            CleaningError::InvalidPath(msg) => 
                write!(f, "Ungültiger Pfad: {}", msg),
        }
    }
}

impl std::error::Error for CleaningError {}

/// Ergebniszusammenfassung mit erweiterten Metriken
#[derive(Debug, Clone)]
pub struct CleaningSummary {
    pub deleted_files: usize,
    pub total_size: u64,
    pub errors: Vec<CleaningError>,
    pub skipped_files: usize,
    pub cleaned_locations: HashMap<String, LocationSummary>,
    pub processing_time: Duration,
    pub empty_dirs_removed: usize,
}

/// Zusammenfassung für einen bestimmten Ort
#[derive(Debug, Clone)]
pub struct LocationSummary {
    pub location_name: String,
    pub deleted_files: usize,
    pub total_size: u64,
    pub errors: usize,
    pub skipped_files: usize,
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
            processing_time: Duration::new(0, 0),
            empty_dirs_removed: 0,
        }
    }

    /// Fügt Daten für einen bestimmten Bereinigungsort hinzu
    pub fn add_location_data(&mut self, location_name: &str, files: usize, size: u64, errors: usize, skipped: usize) {
        self.cleaned_locations.insert(
            location_name.to_string(),
            LocationSummary {
                location_name: location_name.to_string(),
                deleted_files: files,
                total_size: size,
                errors,
                skipped_files: skipped,
            },
        );
    }

    /// Fügt einen Fehler hinzu
    pub fn add_error(&mut self, error: CleaningError) {
        self.errors.push(error);
    }

    /// Menschlesbare Dateigröße
    pub fn formatted_size(&self) -> String {
        format_bytes(self.total_size)
    }

    /// Erfolgsrate in Prozent
    pub fn success_rate(&self) -> f64 {
        let total_processed = self.deleted_files + self.skipped_files + self.errors.len();
        if total_processed == 0 {
            100.0
        } else {
            (self.deleted_files as f64 / total_processed as f64) * 100.0
        }
    }
}

/// Erweiterte Optionen für die Bereinigung
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
    /// Maximale Dateigröße in Bytes (0 = unbegrenzt)
    pub max_file_size: u64,
    /// Minimale Dateigröße in Bytes (0 = alle)
    pub min_file_size: u64,
    /// Ausgeschlossene Dateinamen/Muster
    pub excluded_patterns: Vec<String>,
    /// Verbose Logging
    pub verbose: bool,
    /// Dry Run (simulieren ohne zu löschen)
    pub dry_run: bool,
}

impl Default for CleaningOptions {
    fn default() -> Self {
        Self {
            min_file_age_days: 1,
            recursive: true,
            remove_empty_dirs: true,
            target_extensions: None,
            max_files: 0,
            max_file_size: 0,
            min_file_size: 0,
            excluded_patterns: Vec::new(),
            verbose: false,
            dry_run: false,
        }
    }
}

/// Browser-spezifische Cache-Informationen
#[derive(Debug)]
struct BrowserCacheInfo {
    name: &'static str,
    base_paths: Vec<String>,
    cache_subdirs: Vec<&'static str>,
}

/// Löscht temporäre Dateien aus standard Verzeichnissen
pub fn clean_temp_files() -> Result<CleaningSummary, String> {
    clean_temp_files_with_options(CleaningOptions::default())
}

/// Löscht temporäre Dateien mit benutzerdefinierten Optionen
pub fn clean_temp_files_with_options(options: CleaningOptions) -> Result<CleaningSummary, String> {
    let start_time = SystemTime::now();
    let mut summary = CleaningSummary::new();

    if options.verbose {
        println!("Starte Temp-Dateien Bereinigung...");
        if options.dry_run {
            println!("DRY RUN Modus - Keine Dateien werden gelöscht");
        }
    }

    // Standard Windows Temp-Verzeichnisse
    let temp_locations = get_all_temp_locations();

    for (location_name, paths) in temp_locations {
        let location_files_before = summary.deleted_files;
        let location_size_before = summary.total_size;
        let location_errors_before = summary.errors.len();
        let location_skipped_before = summary.skipped_files;
        
        for path in paths {
            if path.exists() {
                if let Err(e) = clean_directory_advanced(&path, &location_name, &mut summary, &options) {
                    summary.add_error(CleaningError::IoError(path.clone(), e));
                }
            } else if options.verbose {
                println!("Verzeichnis nicht gefunden: {}", path.display());
            }
        }
        
        // Statistiken für diesen Standort speichern
        let files_cleaned = summary.deleted_files - location_files_before;
        let size_cleaned = summary.total_size - location_size_before;
        let errors_count = summary.errors.len() - location_errors_before;
        let skipped_count = summary.skipped_files - location_skipped_before;
        
        if files_cleaned > 0 || errors_count > 0 || skipped_count > 0 {
            summary.add_location_data(&location_name, files_cleaned, size_cleaned, errors_count, skipped_count);
        }

        if options.verbose {
            println!("{}: {} Dateien gelöscht, {} übersprungen, {} Fehler", 
                location_name, files_cleaned, skipped_count, errors_count);
        }
    }

    // Browser-spezifische Bereinigung
    clean_browser_caches(&mut summary, &options)?;

    if let Ok(elapsed) = start_time.elapsed() {
        summary.processing_time = elapsed;
    }

    if options.verbose {
        println!("Bereinigung abgeschlossen in {:?}", summary.processing_time);
        println!("Insgesamt: {} Dateien gelöscht, {} übersprungen, {} Fehler", 
            summary.deleted_files, summary.skipped_files, summary.errors.len());
    }

    Ok(summary)
}

/// Erweiterte Verzeichnisbereinigung mit verbesserter Performance
fn clean_directory_advanced(
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

    let mut directories_to_process = Vec::new();

    // Direkte Verarbeitung der Einträge für bessere Performance
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
        Err(e) => {
            summary.add_error(CleaningError::IoError(dir.to_path_buf(), e.to_string()));
            continue;
        }
        };
        
        let path = entry.path();
        
        // Maximale Dateienanzahl prüfen
        if options.max_files > 0 && summary.deleted_files >= options.max_files {
            break;
        }

        if path.is_file() {
            process_file(&path, location_name, summary, options);
        } else if path.is_dir() && options.recursive {
            // Symlinks überspringen
            let is_symlink = fs::symlink_metadata(&path)
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false);
            
            if !is_symlink {
                directories_to_process.push(path);
            } else {
                summary.skipped_files += 1;
            }
        }
    }

    // Unterverzeichnisse rekursiv verarbeiten
    for dir_path in directories_to_process {
        clean_directory_advanced(&dir_path, location_name, summary, options)?;
    }

    // Leere Verzeichnisse entfernen
    if options.remove_empty_dirs {
        remove_empty_directories_safe(dir, summary, options)?;
    }

    Ok(())
}

/// Verbesserte Dateiverarbeitung
fn process_file(
    path: &Path,
    location_name: &str,
    summary: &mut CleaningSummary,
    options: &CleaningOptions
) {
    // Überspringe spezielle Systemdateien
    if should_skip_file_advanced(path, location_name, options) {
        summary.skipped_files += 1;
        return;
    }

    // Überprüfe Dateiendung, falls gewünscht
    if let Some(extensions) = &options.target_extensions {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                summary.skipped_files += 1;
                return;
            }
        } else {
            summary.skipped_files += 1;
            return;
        }
    }

    // Dateigröße prüfen
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            summary.add_error(CleaningError::IoError(path.to_path_buf(), e.to_string()));
            return;
        }
    };

    let file_size = metadata.len();

    // Größenfilter anwenden
    if options.max_file_size > 0 && file_size > options.max_file_size {
        summary.skipped_files += 1;
        return;
    }

    if options.min_file_size > 0 && file_size < options.min_file_size {
        summary.skipped_files += 1;
        return;
    }

    // Nur Dateien löschen, die älter als die angegebene Zeit sind
    if !is_file_older_than_days(path, options.min_file_age_days).unwrap_or(false) {
        summary.skipped_files += 1;
        return;
    }

    // Datei löschen oder simulieren
    if options.dry_run {
        if options.verbose {
            println!("DRY RUN: Würde löschen: {}", path.display());
        }
        summary.deleted_files += 1;
        summary.total_size += file_size;
    } else {
        match fs::remove_file(path) {
            Ok(()) => {
                summary.deleted_files += 1;
                summary.total_size += file_size;
                if options.verbose {
                    println!("Gelöscht: {}", path.display());
                }
            },
            Err(e) => {
                if is_file_in_use_error(&e) {
                    summary.add_error(CleaningError::FileInUse(path.to_path_buf()));
                } else if e.kind() == io::ErrorKind::PermissionDenied {
                    summary.add_error(CleaningError::PermissionDenied(path.to_path_buf()));
                } else {
                    summary.add_error(CleaningError::IoError(path.to_path_buf(), e.to_string()));
                }
                summary.skipped_files += 1;
            }
        }
    }
}

/// Erweiterte Dateifilterung mit Pattern-Matching
fn should_skip_file_advanced(path: &Path, location_name: &str, options: &CleaningOptions) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    // Versteckte Dateien überspringen
    if file_name.starts_with('.') {
        return true;
    }

    // Benutzerdefinierte Ausschlussmuster
    for pattern in &options.excluded_patterns {
        if matches_pattern(file_name, pattern) {
            return true;
        }
    }
    
    // Location-spezifische geschützte Dateien
    let protected_files: &[&str] = match location_name {
        "Windows Temp" => &["desktop.ini", "thumbs.db", "*.log"],
        "Benutzer Temp" => &["desktop.ini", "some_app_config"],
        "Windows Prefetch" => &["layout.ini", "pf.txt"],
        "Internet Explorer Cache" => &["index.dat", "*.dat"],
        "Chrome Cache" => &["index", "data_*"],
        "Firefox Cache" => &["places.sqlite", "cookies.sqlite", "*.sqlite-wal", "*.sqlite-shm"],
        "Edge Cache" => &["index", "data_*"],
        "Brave Cache" => &["index", "data_*"],
        "Windows Miniaturansichten" => &["thumbcache_*.db", "iconcache_*.db"],
        "Windows Update Cache" => &["*.cab", "*.msu"],
        _ => &["desktop.ini", "thumbs.db", "ntuser.dat", "iconcache.db", "bootsect.bak", "pagefile.sys"],
    };
    
    protected_files.iter().any(|pattern| matches_pattern(file_name, pattern))
}

/// Pattern-Matching mit Wildcard-Unterstützung
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        match parts.len() {
            1 => filename.eq_ignore_ascii_case(pattern),
            2 => {
                filename.to_lowercase().starts_with(&parts[0].to_lowercase()) && 
                filename.to_lowercase().ends_with(&parts[1].to_lowercase())
            },
            _ => {
                // Komplexere Wildcard-Muster (vereinfacht)
                let pattern_lower = pattern.to_lowercase();
                let filename_lower = filename.to_lowercase();
                
                let mut pattern_chars = pattern_lower.chars().peekable();
                let mut filename_chars = filename_lower.chars().peekable();
                
                while let Some(p_char) = pattern_chars.next() {
                    if p_char == '*' {
                        if pattern_chars.peek().is_none() {
                            return true; // Pattern endet mit *, passt zu allem
                        }
                        // Vereinfachte Wildcard-Logik
                        while filename_chars.peek().is_some() {
                            let remaining_filename: String = filename_chars.clone().collect();
                            let remaining_pattern: String = pattern_chars.clone().collect();
                            if matches_pattern(&remaining_filename, &remaining_pattern) {
                                return true;
                            }
                            filename_chars.next();
                        }
                        return false;
                    } else if let Some(f_char) = filename_chars.next() {
                        if p_char != f_char {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                filename_chars.peek().is_none()
            }
        }
    } else {
        filename.eq_ignore_ascii_case(pattern)
    }
}

/// Sichere Entfernung leerer Verzeichnisse
fn remove_empty_directories_safe(
    dir: &Path,
    summary: &mut CleaningSummary,
    options: &CleaningOptions
) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()), // Verzeichnis ist möglicherweise bereits gelöscht
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Fehler beim Lesen des Eintrags: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            // Rekursiv in Unterverzeichnisse
            remove_empty_directories_safe(&path, summary, options)?;
            
            // Prüfen, ob Verzeichnis jetzt leer ist
            if is_directory_empty(&path).unwrap_or(false) {
                if options.dry_run {
                    if options.verbose {
                        println!("DRY RUN: Würde leeres Verzeichnis löschen: {}", path.display());
                    }
                    summary.empty_dirs_removed += 1;
                } else {
                    match fs::remove_dir(&path) {
                        Ok(()) => {
                            summary.empty_dirs_removed += 1;
                            if options.verbose {
                                println!("Leeres Verzeichnis entfernt: {}", path.display());
                            }
                        },
                        Err(e) => {
                            summary.add_error(CleaningError::IoError(path, e.to_string()));
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Alle Temp-Verzeichnisse mit erweiterten Browser-Caches
fn get_all_temp_locations() -> HashMap<String, Vec<PathBuf>> {
    let mut locations = HashMap::new();
    
    // Standard Windows Temp-Verzeichnisse
    if let Some(dir) = get_windows_temp_dir() {
        locations.insert("Windows Temp".to_string(), vec![dir]);
    }
    
    if let Some(dir) = get_user_temp_dir() {
        locations.insert("Benutzer Temp".to_string(), vec![dir]);
    }
    
    if let Some(dir) = get_prefetch_dir() {
        locations.insert("Windows Prefetch".to_string(), vec![dir]);
    }
    
    if let Some(dir) = get_ie_cache_dir() {
        locations.insert("Internet Explorer Cache".to_string(), vec![dir]);
    }
    
    if let Some(dir) = get_thumbnails_cache_dir() {
        locations.insert("Windows Miniaturansichten".to_string(), vec![dir]);
    }
    
    if let Some(dir) = get_windows_update_dir() {
        locations.insert("Windows Update Cache".to_string(), vec![dir]);
    }

    // Zusätzliche Windows-spezifische Temp-Verzeichnisse
    if let Some(dirs) = get_additional_windows_temp_dirs() {
        locations.insert("Weitere Windows Temp".to_string(), dirs);
    }

    // System-spezifische Temp-Verzeichnisse
    if let Some(dirs) = get_system_temp_dirs() {
        locations.insert("System Temp".to_string(), dirs);
    }
    
    locations
}

/// Browser-spezifische Cache-Bereinigung
fn clean_browser_caches(summary: &mut CleaningSummary, options: &CleaningOptions) -> Result<(), String> {
    let browsers = get_browser_cache_info();
    
    for browser in browsers {
        let location_files_before = summary.deleted_files;
        let location_size_before = summary.total_size;
        let location_errors_before = summary.errors.len();
        let location_skipped_before = summary.skipped_files;
        
        for base_path_template in &browser.base_paths {
            if let Some(base_path) = expand_environment_path(base_path_template) {
                for cache_subdir in &browser.cache_subdirs {
                    let cache_path = base_path.join(cache_subdir);
                    if cache_path.exists() {
                        if let Err(e) = clean_directory_advanced(&cache_path, browser.name, summary, options) {
                            summary.add_error(CleaningError::IoError(cache_path, e));
                        }
                    }
                }
            }
        }
        
        // Statistiken für diesen Browser
        let files_cleaned = summary.deleted_files - location_files_before;
        let size_cleaned = summary.total_size - location_size_before;
        let errors_count = summary.errors.len() - location_errors_before;
        let skipped_count = summary.skipped_files - location_skipped_before;
        
        if files_cleaned > 0 || errors_count > 0 || skipped_count > 0 {
            summary.add_location_data(browser.name, files_cleaned, size_cleaned, errors_count, skipped_count);
        }
    }
    
    Ok(())
}

/// Browser-Cache-Informationen mit erweiterten Pfaden
fn get_browser_cache_info() -> Vec<BrowserCacheInfo> {
    vec![
        BrowserCacheInfo {
            name: "Chrome Cache",
            base_paths: vec![
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default".to_string(),
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Profile 1".to_string(),
                "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Profile 2".to_string(),
            ],
            cache_subdirs: vec!["Cache", "GPUCache", "Code Cache", "Service Worker\\CacheStorage"],
        },
        BrowserCacheInfo {
            name: "Firefox Cache",
            base_paths: get_firefox_profile_paths(),
            cache_subdirs: vec!["cache2", "startupCache", "OfflineCache", "thumbnails"],
        },
        BrowserCacheInfo {
            name: "Edge Cache",
            base_paths: vec![
                "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default".to_string(),
                "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Profile 1".to_string(),
            ],
            cache_subdirs: vec!["Cache", "GPUCache", "Code Cache", "Service Worker\\CacheStorage"],
        },
        BrowserCacheInfo {
            name: "Brave Cache",
            base_paths: vec![
                "%LOCALAPPDATA%\\BraveSoftware\\Brave-Browser\\User Data\\Default".to_string(),
                "%LOCALAPPDATA%\\BraveSoftware\\Brave-Browser\\User Data\\Profile 1".to_string(),
            ],
            cache_subdirs: vec!["Cache", "GPUCache", "Code Cache"],
        },
        BrowserCacheInfo {
            name: "Opera Cache",
            base_paths: vec![
                "%APPDATA%\\Opera Software\\Opera Stable".to_string(),
                "%APPDATA%\\Opera Software\\Opera GX Stable".to_string(),
            ],
            cache_subdirs: vec!["Cache", "GPUCache", "Code Cache"],
        },
        BrowserCacheInfo {
            name: "Vivaldi Cache",
            base_paths: vec![
                "%LOCALAPPDATA%\\Vivaldi\\User Data\\Default".to_string(),
            ],
            cache_subdirs: vec!["Cache", "GPUCache", "Code Cache"],
        },
    ]
}

/// Firefox-Profile dynamisch ermitteln
fn get_firefox_profile_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    if let Ok(appdata) = env::var("APPDATA") {
        let profiles_dir = PathBuf::from(appdata).join("Mozilla\\Firefox\\Profiles");
        if let Ok(entries) = fs::read_dir(&profiles_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    paths.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    
    if paths.is_empty() {
        paths.push("%APPDATA%\\Mozilla\\Firefox\\Profiles\\*".to_string());
    }
    
    paths
}

/// Umgebungsvariablen in Pfaden expandieren
fn expand_environment_path(path_template: &str) -> Option<PathBuf> {
    let mut expanded = path_template.to_string();
    
    // Ersetze Umgebungsvariablen
    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        expanded = expanded.replace("%LOCALAPPDATA%", &localappdata);
    }
    if let Ok(appdata) = env::var("APPDATA") {
        expanded = expanded.replace("%APPDATA%", &appdata);
    }
    if let Ok(userprofile) = env::var("USERPROFILE") {
        expanded = expanded.replace("%USERPROFILE%", &userprofile);
    }
    
    // Wildcard-Unterstützung für Profile
    if expanded.contains('*') {
        let parent = PathBuf::from(&expanded.replace("\\*", ""));
        if let Ok(entries) = fs::read_dir(&parent) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return Some(entry.path());
                }
            }
        }
        return None;
    }
    
    Some(PathBuf::from(expanded))
}

/// Zusätzliche Windows Temp-Verzeichnisse
fn get_additional_windows_temp_dirs() -> Option<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    
    // Windows Installer Cache
    if let Ok(windows) = env::var("SystemRoot") {
        let windows_path = PathBuf::from(&windows);
        dirs.push(windows_path.join("Installer\\$PatchCache$"));
        dirs.push(windows_path.join("SoftwareDistribution\\DataStore"));
        dirs.push(windows_path.join("Logs"));
    }
    
    // IIS Logs
    if let Some(iis_logs) = env::var("SystemDrive").ok().map(|d| PathBuf::from(d).join("inetpub\\logs")) {
        if iis_logs.exists() {
            dirs.push(iis_logs);
        }
    }
    
    if dirs.is_empty() { None } else { Some(dirs) }
}

/// System-spezifische Temp-Verzeichnisse
fn get_system_temp_dirs() -> Option<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    
    // Benutzer-spezifische Temp-Verzeichnisse
    if let Ok(userprofile) = env::var("USERPROFILE") {
        let user_path = PathBuf::from(userprofile);
        dirs.push(user_path.join("AppData\\Local\\Temp"));
        dirs.push(user_path.join("AppData\\Local\\Microsoft\\Windows\\INetCache"));
        dirs.push(user_path.join("AppData\\Local\\Microsoft\\Windows\\WebCache"));
        dirs.push(user_path.join("AppData\\Roaming\\Microsoft\\Windows\\Recent"));
    }
    
    if dirs.is_empty() { None } else { Some(dirs) }
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

/// Verbesserte Prüfung für "Datei in Benutzung" Fehler mit Windows-Konstanten
#[cfg(windows)]
fn is_file_in_use_error(error: &io::Error) -> bool {
    match error.raw_os_error() {
        Some(err) if err == ERROR_SHARING_VIOLATION.0 as i32 => true,
        Some(err) if err == ERROR_LOCK_VIOLATION.0 as i32 => true,
        _ => error.kind() == io::ErrorKind::PermissionDenied,
    }
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
    
    if size < 10.0 && unit_idx > 0 {
        format!("{:.2} {}", size, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.0 Bytes");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("test.txt", "*.txt"));
        assert!(matches_pattern("thumbcache_1024.db", "thumbcache_*.db"));
        assert!(!matches_pattern("test.log", "*.txt"));
        assert!(matches_pattern("test.txt", "test.txt"));
    }

    #[test]
    fn test_cleaning_summary() {
        let mut summary = CleaningSummary::new();
        assert_eq!(summary.deleted_files, 0);
        assert_eq!(summary.total_size, 0);
        assert_eq!(summary.success_rate(), 100.0);
        
        summary.deleted_files = 10;
        summary.skipped_files = 5;
        summary.add_error(CleaningError::PermissionDenied(PathBuf::from("test")));
        
        assert!(summary.success_rate() > 60.0 && summary.success_rate() < 70.0);
    }

    #[test]
    fn test_temp_directory_cleaning() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Erstelle Testdateien
        let test_file = temp_path.join("test.tmp");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "test content").unwrap();
        
        let mut options = CleaningOptions::default();
        options.min_file_age_days = 0; // Alle Dateien löschen
        options.dry_run = true; // Nur simulieren
        
        let mut summary = CleaningSummary::new();
        
        // Test der Verzeichnisbereinigung
        let result = clean_directory_advanced(temp_path, "Test", &mut summary, &options);
        assert!(result.is_ok());
    }
}
