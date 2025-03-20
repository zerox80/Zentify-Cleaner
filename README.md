# RustyClean

**RustyClean** ist ein umfassendes Open-Source-Tool zur Optimierung der Systemleistung unter Windows 11. Neben der klassischen Reinigung von temporären Dateien bietet RustyClean zahlreiche Funktionen, um Windows auf mehreren Ebenen zu optimieren – von der Bereinigung und Performance-Verbesserung über das Ressourcenmanagement bis hin zur individuellen Systemanpassung.

## Überblick

RustyClean bietet ein modulares Konzept, das es ermöglicht, verschiedene Aspekte der Systemoptimierung anzugehen. Es kombiniert bewährte Reinigungstechniken (wie das Entfernen temporärer Dateien, Log-Dateien und Browser-Caches) mit erweiterten Windows-spezifischen Optimierungen, um die Gesamtleistung des Systems zu steigern. Die Anwendung ist in Rust geschrieben, nutzt das moderne GUI-Framework [iced](https://github.com/iced-rs/iced) und ist als Plattform für zukünftige Erweiterungen konzipiert.

## Hauptfunktionen

### Systemreinigung

- **Temporäre Dateien löschen:**  
  ✅ Rekursive Suche und Entfernung von temporären Dateien in `%TEMP%`, `C:\Windows\Temp` und weiteren benutzerdefinierten Ordnern.

- **Log-Dateien bereinigen:**  
  Automatisches Erkennen und Löschen veralteter oder fehlerhafter Log-Dateien aus System- und Anwendungsordnern.

- **Browser-Cache und History löschen:**  
  Unterstützung beim Entfernen von Caches, temporären Dateien und Verlaufseinträgen gängiger Browser wie Chrome, Firefox und Edge.

### Windows-spezifische Optimierungen

- **Registry-Cleanup:**  
  Analyse und sichere Bereinigung veralteter oder ungültiger Registry-Einträge, um Systemfehler zu minimieren und die Stabilität zu erhöhen.  
  *Hinweis: Diese Funktion erfordert besondere Vorsicht und sollte nur nach Rücksprache mit dem Benutzer ausgeführt werden.*

- **Autostart-Manager:**  
  Übersicht und Verwaltung von Programmen und Diensten, die beim Systemstart automatisch geladen werden. Ermögliche das Deaktivieren unnötiger Anwendungen, um den Bootvorgang zu beschleunigen.

- **Dienst- und Service-Optimierung:**  
  Ermöglicht das Überwachen und Anpassen von Windows-Diensten. Nicht benötigte Dienste können vorübergehend deaktiviert werden, um Systemressourcen zu schonen.

- **Defragmentierung und SSD-Optimierung:**  
  Integration von Windows-eigenen Optimierungswerkzeugen zur Defragmentierung von HDDs und TRIM-Befehlen für SSDs. Zeigt Empfehlungen zur regelmäßigen Wartung an.

- **Visuelle Effekte anpassen:**  
  Bietet eine Schnittstelle, um visuelle Effekte (Animationen, Transparenzen) zu reduzieren oder anzupassen, sodass mehr Systemressourcen für wichtige Aufgaben verfügbar sind.

- **Energie- und Leistungseinstellungen:**  
  Ermöglicht das Umschalten zwischen verschiedenen Windows-Energieplänen (z. B. "Höchstleistung", "Ausbalanciert") und optimiert die Einstellungen für maximale Performance.

### Ressourcenmanagement und Überwachung

- **Systemüberwachung:**  
  Echtzeit-Überwachung von CPU-, Speicher- und Festplattennutzung. Identifiziert Engpässe und bietet Empfehlungen zur Optimierung.

- **Prozessoptimierung:**  
  Analyse der laufenden Prozesse mit Vorschlägen, welche Anwendungen oder Hintergrunddienste beendet oder priorisiert werden sollten, um die Leistung zu steigern.

- **Cache-Management:**  
  Erweiterte Routinen zur Bereinigung diverser Caches, wie System-, Anwendungs- und DNS-Cache, um den Speicherplatz effizient zu nutzen.

### Erweiterte Systemanpassung

- **Zeitgesteuerte Reinigung:**  
  Plane regelmäßige Reinigungs- und Optimierungsvorgänge, um das System kontinuierlich in einem optimalen Zustand zu halten.

- **Detaillierte Leistungsberichte:**  
  Erstelle vor und nach der Optimierung umfassende Berichte über Systemressourcen, verbrauchten Speicherplatz und die erreichte Performance-Verbesserung.

- **Anpassbare Reinigungs- und Optimierungsprofile:**  
  Erstelle individuelle Profile, mit denen spezifische Bereiche des Systems (wie Startprogramme, Dienste oder visuelle Effekte) gezielt optimiert werden können.

- **Backup und Wiederherstellung:**  
  Vor kritischen Operationen, wie dem Registry-Cleanup, wird ein automatisches Backup erstellt. Bei Problemen können Änderungen rückgängig gemacht werden.

- **Netzwerkoptimierung:**  
  Überwache die Netzwerkleistung und biete Optionen zur Optimierung der Netzwerkeinstellungen (z. B. TCP/IP-Konfiguration und DNS-Cache), um eine bessere Internet- und Intranet-Performance zu erzielen.

- **Treiber- und Softwaremanagement:**  
  Bietet Empfehlungen für veraltete Treiber oder Softwarekomponenten, um Kompatibilitätsprobleme zu vermeiden und die Systemstabilität zu verbessern.

## Technologien

- **Programmiersprache:** Rust – für hohe Performance, Systemsicherheit und Effizienz.
- **GUI-Framework:** [iced](https://github.com/iced-rs/iced) – für eine moderne, responsive Benutzeroberfläche.
- **Dateisystemoperationen:** Rusts Standardbibliothek sowie Crates wie [directories](https://crates.io/crates/directories) für plattformübergreifende Verzeichnisermittlung.
- **Logging und Monitoring:** Einsatz von Crates wie `log` und `env_logger` für detaillierte Protokollierung und Diagnose.

## Voraussetzungen

- **Betriebssystem:** Windows 11 (Administratorrechte können für bestimmte Funktionen erforderlich sein).
- **Rust:** Aktuelle stabile Version (Installation über [rustup](https://www.rust-lang.org/tools/install)).
- **Git:** Für das Klonen und Verwalten des Repositories.

## Installation

1. **Rust und Git installieren:**  
   Lade Rust und Git von den offiziellen Seiten herunter:
   - [Rust Installation](https://www.rust-lang.org/tools/install)
   - [Git Download](https://git-scm.com/downloads)

2. **Repository klonen:**  
   Öffne eine Eingabeaufforderung oder PowerShell und führe aus:
   ```bash
   git clone https://github.com/dein-benutzername/rustyclean.git
   cd rustyclean
   ```

3. **Kompilieren und Ausführen:**
   ```bash
   cargo build --release
   cargo run --release
   ```

## Funktionen

- **✅ Temporäre Dateien löschen:** Bereinigt Windows-Temp-Verzeichnisse (`%TEMP%`, `C:\Windows\Temp`).
- *(Weitere Funktionen werden in zukünftigen Updates hinzugefügt)*

## Lizenz

MIT 