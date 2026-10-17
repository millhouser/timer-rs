use std::path::{Path, PathBuf};

pub async fn choose_sound(directory: &Path) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Sound-Datei")
        .set_directory(directory)
        .add_filter("Wave-Dateien", &["wav"])
        .pick_file()
        .await
        .map(|file| file.path().to_owned())
}

pub fn notify(message: &str, warning: bool) {
    let message = message.to_owned();
    std::thread::spawn(move || {
        if let Err(error) = notify_rust::Notification::new()
            .summary("Timer")
            .body(&message)
            .urgency(if warning {
                notify_rust::Urgency::Critical
            } else {
                notify_rust::Urgency::Normal
            })
            .show()
        {
            eprintln!("Timer: {message} ({error})");
        }
    });
}

pub fn local_time() -> String {
    chrono::Local::now().format("%H:%M Uhr").to_string()
}

#[cfg(windows)]
pub fn tray_available() -> bool {
    true
}

#[cfg(target_os = "linux")]
pub fn tray_available() -> bool {
    let Ok(connection) = zbus::blocking::Connection::session() else {
        return false;
    };
    let Ok(watcher) = zbus::blocking::Proxy::new(
        &connection,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        "org.kde.StatusNotifierWatcher",
    ) else {
        return false;
    };
    watcher
        .get_property::<bool>("IsStatusNotifierHostRegistered")
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn play_sound(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME};
    let filename: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        PlaySoundW(
            filename.as_ptr(),
            std::ptr::null_mut(),
            SND_FILENAME | SND_ASYNC,
        );
    }
}

#[cfg(target_os = "linux")]
pub fn play_sound(path: &Path) {
    use std::process::{Command, Stdio};
    let path = path.to_owned();
    std::thread::spawn(move || {
        for player in ["pw-play", "paplay", "aplay"] {
            if Command::new(player)
                .arg(&path)
                .stdin(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
            {
                return;
            }
        }
        notify(
            "Sound konnte nicht abgespielt werden. Benoetigt wird pw-play, paplay oder aplay.",
            true,
        );
    });
}
