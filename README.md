# Zentify-Cleaner

**Zentify-Cleaner** ist ein leistungsstarkes Open-Source-Tool zur Systemoptimierung für Windows, geschrieben in Rust. Mit seiner modernen GUI auf Basis des [iced](https://github.com/iced-rs/iced)-Frameworks ermöglicht der Cleaner eine einfache und effektive Bereinigung temporärer Dateien sowie die Überwachung von Systemressourcen.

![Zentify-Cleaner Screenshot](https://i.imgur.com/bLBY1Nl.png)

## Hauptfunktionen

### Implementierte Funktionen

#### Temporäre Dateien Bereinigung ✅
- **Umfassende Reinigung** in mehreren Systemverzeichnissen:
  - Windows Temp-Verzeichnis
  - Benutzer-Temp-Verzeichnis
  - Windows Prefetch
  - Browser-Caches (Internet Explorer, Chrome, Firefox, Edge, Brave)
  - Windows Miniaturansichten
  - Windows Update-Cache

- **Intelligente Optionen**:
  - Altersfilterung (nur Dateien älter als X Tage)
  - Rekursives oder nicht-rekursives Löschen
  - Automatisches Entfernen leerer Verzeichnisse
  - Dateiendungs-spezifische Bereinigung
  - Begrenzung der maximalen zu löschenden Dateien

#### Echtzeit-Systemüberwachung ✅
- Überwachung von CPU-Auslastung, Arbeitsspeicher und Festplattennutzung
- Anzeige der Top-Prozesse nach Ressourcenverbrauch
- Aktualisierung in Echtzeit

### Vorteile

- **Hohe Performance**: Implementiert in Rust für schnelle Ausführung und minimalen Ressourcenverbrauch
- **Benutzerfreundlich**: Moderne, intuitive Oberfläche 
- **Sicher**: Intelligente Schutzfunktionen für Systemdateien
- **Leichtgewichtig**: Minimaler Speicherbedarf
- **Open Source**: Transparente Funktionsweise und anpassbar

## Voraussetzungen

- **Betriebssystem**: Windows 10/11
- **Administratorrechte**: Für volle Funktionalität empfohlen

## Installation

### Vorcompilierte Version

1. Lade die neueste Version von der [Releases-Seite](https://github.com/dein-benutzername/Zentify-Cleaner/releases) herunter
2. Entpacke die ZIP-Datei an deinen gewünschten Ort
3. Starte `zentify-cleaner.exe` (ggf. als Administrator)

### Aus dem Quellcode

1. Stelle sicher, dass [Rust](https://www.rust-lang.org/tools/install) installiert ist
2. Klone das Repository:
   ```
   git clone https://github.com/dein-benutzername/Zentify-Cleaner.git
   cd Zentify-Cleaner
   ```
3. Kompiliere und starte das Projekt:
   ```
   cargo build --release
   cargo run --release
   ```

## Verwendung

1. Starte die Anwendung
2. Klicke auf "Jetzt bereinigen" im Abschnitt "Temporäre Dateien"
3. Aktiviere die Systemüberwachung mit dem entsprechenden Button

## Beispiel: Benutzerdefinierte Bereinigung

```rust
// Spezifische Reinigungsoptionen erstellen
let options = CleaningOptions {
    min_file_age_days: 7,               // Nur Dateien älter als eine Woche
    recursive: true,                     // Auch Unterordner bereinigen
    remove_empty_dirs: true,             // Leere Ordner löschen
    target_extensions: Some(vec![        // Nur bestimmte Dateiendungen löschen
        "tmp".to_string(), 
        "log".to_string(),
        "bak".to_string()
    ]),
    max_files: 1000,                     // Maximal 1000 Dateien pro Durchgang
};

// Bereinigung mit benutzerdefinierten Optionen durchführen
let summary = clean_temp_files_with_options(options)?;
println!("Bereinigt: {} Dateien ({})", summary.deleted_files, summary.formatted_size());
```

## Geplante Funktionen

- [ ] Browser-History-Bereinigung 
- [ ] Autostart-Manager
- [ ] Registry-Optimierung
- [ ] Erweiterte Reinigungsprofile
- [ ] Zeitgesteuerte Reinigungen

## Mitwirken

Beiträge sind herzlich willkommen! Bitte beachte folgende Schritte:

1. Fork das Repository
2. Erstelle einen Feature-Branch (`git checkout -b feature/AmazingFeature`)
3. Commit deine Änderungen (`git commit -m 'Add some AmazingFeature'`)
4. Push zum Branch (`git push origin feature/AmazingFeature`)
5. Öffne einen Pull Request

## Lizenz

Dieses Projekt ist unter der MIT-Lizenz veröffentlicht. Siehe [LICENSE](LICENSE) für Details.

## Kontakt
rujbin[at]proton[dot]me
