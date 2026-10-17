# Timer (Rust + Slint)

Port des Qt-Projekts aus `../timer` fuer Windows und Linux, mit Slint 1.18.1
fuer Fenster, Dialoge, Tray-Icon und Tray-Menues.

## Voraussetzungen

- Rust mit Cargo
- Windows: Windows 10/11, MSVC-Toolchain, Visual Studio Build Tools mit
  C++-Werkzeugen und Windows SDK
- Linux: Rust-Toolchain und C/C++-Linker; zum Starten eine grafische Sitzung
  mit Wayland sowie Fontconfig- und XKB-Laufzeitbibliotheken
  (`libfontconfig.so.1`, `libxkbcommon.so.0`). X11 wird nicht unterstuetzt.

Fontconfig wird unter Linux dynamisch geladen. Fontconfig-Entwicklungspakete,
ein angepasstes `PKG_CONFIG_PATH` oder besondere Cargo-Features sind nicht noetig.
Die Slint-Versionen fuer Laufzeit und Build sind exakt auf 1.18.1 festgelegt.

## Bauen und starten

Auf beiden Plattformen unveraendert:

```sh
cargo build --release
cargo run --release
cargo test
```

Cargo waehlt ueber `.cargo/config.toml` automatisch die Host-Plattform als
Build-Ziel. Windows- und Linux-Artefakte liegen dadurch getrennt, zum Beispiel:

- Windows (GNU): `target/x86_64-pc-windows-gnu/release/timer-rs.exe`
- Windows (MSVC): `target/x86_64-pc-windows-msvc/release/timer-rs.exe`
- Linux: `target/x86_64-unknown-linux-gnu/release/timer-rs`

`ding.wav` wird beim Build automatisch daneben kopiert. Debug-Builds liegen
entsprechend im jeweiligen `debug`-Unterordner. Die Plattformbezeichnung folgt
automatisch der installierten Rust-Toolchain. Cargo kann weiterhin gemeinsame
Host-Build-Werkzeuge unter `target` zwischenspeichern. Bestehende Dateien in den
alten Ausgabeordnern werden nicht automatisch geloescht.

Bei verfuegbarem Infobereich startet die Anwendung wie das Original dort.
Rechtsklick auf das Uhrsymbol oeffnet das Menue, ein Doppelklick startet die
zuletzt gewaehlte Dauer erneut, auch bei bereits laufendem Timer. Unter Linux
haengen Klickgesten vom Desktop ab; **Neu starten** ist auch im Menue verfuegbar.
Unter **Einstellungen...** stehen WAV-Datei, Tonvorschau, Sound-Schalter
und die benutzerdefinierte Dauer (1 bis 99 Minuten, Vorgabe 11) bereit.
Schliessen und **OK** verbergen den Dialog; nur **Beenden** beendet das Programm.
**OK** prueft die Sound-Datei auch bei ausgeschalteter Wiedergabe.

### Linux und WSL

Unter Linux wird ausschliesslich Slints Wayland-Backend gebaut. Eine
Backend-Umgebungsvariable oder ein X11-Fallback ist nicht vorgesehen.

Slints Tray benoetigt einen StatusNotifierItem-/AppIndicator-Host. GNOME
benoetigt dafuer gegebenenfalls eine AppIndicator-Erweiterung. Ohne Tray-Host,
etwa unter WSLg, wird stattdessen ein Timer-Fenster mit Start, Neustart,
Abbruch und Restzeit angezeigt. Dessen Schliessen beendet die Anwendung;
**OK** laesst das Fenster geoeffnet.

Die native Dateiauswahl verwendet unter Linux den XDG-Desktop-Portal-Dienst
(mit passendem Desktop-Backend). Der Dateipfad kann auch direkt eingegeben
werden. Systembenachrichtigungen benoetigen einen Benachrichtigungsdienst.
Fuer WAV-Wiedergabe muss mindestens einer der Desktop-Player `pw-play`,
`paplay` oder `aplay` verfuegbar sein, beispielsweise aus `pipewire-bin`,
`pulseaudio-utils` oder `alsa-utils`. Fehlende Wiedergabe wird gemeldet.
Diese Dienste und Programme werden nicht zum Kompilieren benoetigt.

### Windows-Deployment

```powershell
.\build.ps1
```

`build.ps1` baut die Release-Version und aktualisiert danach automatisch
`deploy/timer-rs.exe` und `deploy/ding.wav`. Zur Weitergabe den gesamten
`deploy`-Ordner verwenden. Schlaegt der Build fehl, bleibt dessen bisheriger
Inhalt unveraendert. Vor einem erneuten Build eine aus `deploy` gestartete
Timer-Instanz beenden, damit Windows die EXE zum Ersetzen freigibt.

Ein direktes `cargo build --release` baut weiterhin nur die Anwendung und
aktualisiert `deploy` nicht. Fuer Build und Deployment daher `build.ps1`
verwenden. Das Uhrsymbol ist eingebettet. Es wird weder Qt noch eine
Slint-Installation auf dem Zielrechner benoetigt.

## Verhalten

- Vordefinierte Timer von 1 bis 10 Minuten und benutzerdefinierte Dauer.
- Laufender Timer: Abbruchmenue, Restzeit im Tooltip, aufgerundete Minuten
  in blauer Schrift auf dem Uhrsymbol.
- Ablauf: Desktop-Benachrichtigung mit lokaler Uhrzeit und optional asynchronem WAV-Sound.
- Ablauf und Abbruch setzen das Uhrsymbol zurueck.
- Einstellungen werden wie im Original nicht gespeichert.
- Windows kann Benachrichtigungen je nach Systemeinstellungen unterdruecken
  oder ihre Anzeigedauer selbst bestimmen.

Die Countdown-Zustandswechsel, Texte und die gemeinsame Icon-Erzeugung
sind durch Unit-Tests abgedeckt.
