use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Löscht temporäre Dateien aus Standardverzeichnissen und weiteren Orten
pub fn clean_temp_files() -> Result<CleaningSummary, String> {
    let mut summary = CleaningSummary {
        deleted_files: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    // Erweiterte Liste von temporären Verzeichnissen
    let temp_directories = vec![
        get_windows_temp_dir(),         // Windows\Temp
        get_user_temp_dir(),            // Benutzer-TEMP
        get_prefetch_dir(),             // Windows\Prefetch (Vorfetch-Dateien)
        get_windows_update_dir(),       // SoftwareDistribution\Download
        get_recycle_bin(),              // Papierkorb
        get_ie_cache_dir(),             // Temporary Internet Files
        get_browser_cache_dir("Chrome"),// Chrome Cache
        get_browser_cache_dir("Edge"),  // Edge Cache
        get_browser_cache_dir("Firefox"),// Firefox Cache (erstes Profil mit cache2)
        get_recent_docs_dir(),          // Recent-Dokumente
    ];

    // Leere die Verzeichnisse
    for dir in temp_directories {
        if let Some(dir_path) = dir {
            if let Err(e) = clean_directory(&dir_path, &mut summary) {
                summary.errors.push(format!("Verzeichnis {}: {}", dir_path.display(), e));
            }
        }
    }

    // Spezielle Dateitypen bereinigen (Log, Dump, TMP-Dateien) im Windows-Verzeichnis
    if let Some(windows_dir) = get_windows_dir() {
        if let Err(e) = clean_file_type(&windows_dir, "*.log", &mut summary) {
            summary.errors.push(format!("Log-Dateien Bereinigung: {}", e));
        }
        if let Err(e) = clean_file_type(&windows_dir
