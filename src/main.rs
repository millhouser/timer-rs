#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod countdown;
mod desktop;
mod tray;

use countdown::{Countdown, minutes_text};
use slint::ComponentHandle;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use tray::Tray;

slint::include_modules!();

struct App {
    countdown: Countdown,
    tray: Tray,
    tick: slint::Timer,
    settings_window: std::rc::Weak<RefCell<Option<SettingsWindow>>>,
    sound_file: String,
    sound_enabled: bool,
}

impl App {
    fn sync_window(&self) {
        if let Some(settings_window) = self.settings_window.upgrade() {
            if let Some(window) = settings_window.borrow().as_ref() {
                window.set_timer_running(self.countdown.running);
                window.set_timer_status(if self.countdown.running {
                    format!(
                        "{}:{:02}",
                        self.countdown.seconds_remaining / 60,
                        self.countdown.seconds_remaining % 60
                    )
                    .into()
                } else {
                    "Timer bereit".into()
                });
            }
        }
    }

    fn start(&mut self, minutes: u32, weak: std::rc::Weak<RefCell<Self>>) {
        if !self.countdown.start(minutes) {
            self.tray.notify("Bitte erst eine Zeit wählen!", false);
            return;
        }
        self.tray.set_running(true);
        self.tray.set_tooltip(&self.countdown.tooltip());
        self.tray.repaint(self.countdown.seconds_remaining);
        self.tick.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                if let Some(app) = weak.upgrade() {
                    app.borrow_mut().timeout();
                }
            },
        );
        self.tray.notify(
            &format!("Timer mit {} gestartet.", minutes_text(minutes)),
            false,
        );
        self.sync_window();
    }

    fn abort(&mut self) {
        self.tick.stop();
        self.countdown.abort();
        self.tray.set_running(false);
        self.tray.set_tooltip(&self.countdown.tooltip());
        self.tray.repaint(0);
        self.tray.notify("Timer abgebrochen!", false);
        self.sync_window();
    }

    fn timeout(&mut self) {
        let expired = self.countdown.tick();
        if expired {
            self.tick.stop();
            self.tray.set_running(false);
            self.tray.set_tooltip(&self.countdown.tooltip());
            self.tray.repaint(0);
            self.tray.notify(
                &format!(
                    "{} abgelaufen!\n\n(Seit {})",
                    minutes_text(self.countdown.last_minutes),
                    desktop::local_time()
                ),
                true,
            );
            self.play();
        } else {
            self.tray.repaint(self.countdown.seconds_remaining);
            self.tray.set_tooltip(&self.countdown.tooltip());
        }
        self.sync_window();
        if expired {
            if let Some(settings_window) = self.settings_window.upgrade() {
                if let Some(window) = settings_window.borrow().as_ref() {
                    window.set_timer_status("Timer abgelaufen!".into());
                }
            }
        }
    }

    fn check_sound(&self, path: &str) -> bool {
        if Path::new(path).exists() {
            true
        } else {
            self.tray
                .notify(&format!("Sound-Datei ({path}) nicht gefunden!"), true);
            false
        }
    }

    fn play(&self) {
        let (path, enabled) = self
            .settings_window
            .upgrade()
            .and_then(|settings_window| {
                settings_window.borrow().as_ref().map(|window| {
                    (
                        window.get_sound_file().to_string(),
                        window.get_sound_enabled(),
                    )
                })
            })
            .unwrap_or_else(|| (self.sound_file.clone(), self.sound_enabled));
        if enabled && self.check_sound(&path) {
            desktop::play_sound(Path::new(&path));
        }
    }
}

fn ensure_settings_window(
    settings_window: &Rc<RefCell<Option<SettingsWindow>>>,
    about_window: &Rc<RefCell<Option<AboutWindow>>>,
    app: &Rc<RefCell<App>>,
    directory: &Path,
) -> Result<(), slint::PlatformError> {
    if settings_window.borrow().is_some() {
        return Ok(());
    }

    let window = SettingsWindow::new()?;
    {
        let app = app.borrow();
        window.set_sound_file(app.sound_file.as_str().into());
        window.set_sound_enabled(app.sound_enabled);
    }
    window.window().on_close_requested({
        let settings_window = Rc::downgrade(settings_window);
        move || {
            let standalone = settings_window
                .upgrade()
                .and_then(|settings_window| {
                    settings_window
                        .borrow()
                        .as_ref()
                        .map(SettingsWindow::get_standalone)
                })
                .unwrap_or(false);
            if standalone {
                let _ = slint::quit_event_loop();
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    window.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
    window.on_about({
        let about_window = about_window.clone();
        move || {
            if let Err(error) = show_about(&about_window) {
                eprintln!("Timer: Info-Fenster konnte nicht angezeigt werden: {error}");
            }
        }
    });
    window.on_start({
        let app = Rc::downgrade(app);
        move |minutes| {
            if let Some(app) = app.upgrade() {
                app.borrow_mut().start(minutes as u32, Rc::downgrade(&app));
            }
        }
    });
    window.on_restart({
        let app = Rc::downgrade(app);
        move || {
            if let Some(app) = app.upgrade() {
                let minutes = app.borrow().countdown.last_minutes;
                app.borrow_mut().start(minutes, Rc::downgrade(&app));
            }
        }
    });
    window.on_abort({
        let app = Rc::downgrade(app);
        move || {
            if let Some(app) = app.upgrade() {
                app.borrow_mut().abort();
            }
        }
    });
    window.on_play({
        let app = Rc::downgrade(app);
        move || {
            if let Some(app) = app.upgrade() {
                app.borrow().play();
            }
        }
    });
    window.on_accept({
        let app = Rc::downgrade(app);
        let settings_window = Rc::downgrade(settings_window);
        move || {
            if let (Some(app), Some(settings_window)) = (app.upgrade(), settings_window.upgrade()) {
                if let Some(window) = settings_window.borrow().as_ref() {
                    if app.borrow().check_sound(&window.get_sound_file())
                        && !window.get_standalone()
                    {
                        let _ = window.hide();
                    }
                }
            }
        }
    });
    window.on_browse({
        let settings_window = Rc::downgrade(settings_window);
        let directory = directory.to_owned();
        move || {
            let directory = directory.clone();
            let settings_window = settings_window.clone();
            let _ = slint::spawn_local(async move {
                let selected = desktop::choose_sound(&directory).await;
                if let (Some(path), Some(settings_window)) = (selected, settings_window.upgrade()) {
                    if let Some(window) = settings_window.borrow().as_ref() {
                        window.set_sound_file(path.to_string_lossy().as_ref().into());
                    }
                }
            });
        }
    });
    window.on_duration_changed({
        let app = Rc::downgrade(app);
        move |minutes| {
            if let Some(app) = app.upgrade() {
                app.borrow().tray.set_custom_minutes(minutes as u32);
            }
        }
    });

    *settings_window.borrow_mut() = Some(window);
    Ok(())
}

fn show_settings(
    settings_window: &Rc<RefCell<Option<SettingsWindow>>>,
    about_window: &Rc<RefCell<Option<AboutWindow>>>,
    app: &Rc<RefCell<App>>,
    directory: &Path,
) -> Result<(), slint::PlatformError> {
    ensure_settings_window(settings_window, about_window, app, directory)?;
    app.borrow().sync_window();
    let settings_window = settings_window.borrow();
    let window = settings_window
        .as_ref()
        .expect("Einstellungen-Fenster wurde soeben erzeugt");
    window.show()?;
    window.window().request_redraw();
    Ok(())
}

fn show_about(about_window: &Rc<RefCell<Option<AboutWindow>>>) -> Result<(), slint::PlatformError> {
    if about_window.borrow().is_none() {
        let about = AboutWindow::new()?;
        about.on_dismiss({
            let about_window = Rc::downgrade(about_window);
            move || {
                if let Some(about_window) = about_window.upgrade() {
                    if let Some(about) = about_window.borrow().as_ref() {
                        let _ = about.hide();
                    }
                }
            }
        });
        *about_window.borrow_mut() = Some(about);
    }

    let about_window = about_window.borrow();
    about_window
        .as_ref()
        .expect("Info-Fenster wurde soeben erzeugt")
        .show()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    let directory: PathBuf = executable.parent().unwrap().into();
    let settings_window = Rc::new(RefCell::new(None));
    let about_window = Rc::new(RefCell::new(None));
    let app = Rc::new(RefCell::new(App {
        countdown: Countdown::default(),
        tray: Tray::new()?,
        tick: slint::Timer::default(),
        settings_window: Rc::downgrade(&settings_window),
        sound_file: directory.join("ding.wav").to_string_lossy().into_owned(),
        sound_enabled: true,
    }));

    app.borrow().tray.ui.on_settings({
        let settings_window = settings_window.clone();
        let about_window = about_window.clone();
        let app = Rc::downgrade(&app);
        let directory = directory.clone();
        move || {
            let Some(app) = app.upgrade() else {
                return;
            };
            if let Err(error) = show_settings(&settings_window, &about_window, &app, &directory) {
                eprintln!("Timer: Einstellungen-Fenster konnte nicht angezeigt werden: {error}");
            }
        }
    });
    app.borrow().tray.ui.on_start({
        let app = Rc::downgrade(&app);
        move |minutes| {
            if let Some(app) = app.upgrade() {
                app.borrow_mut().start(minutes as u32, Rc::downgrade(&app));
            }
        }
    });
    app.borrow().tray.ui.on_restart({
        let app = Rc::downgrade(&app);
        move || {
            if let Some(app) = app.upgrade() {
                let minutes = app.borrow().countdown.last_minutes;
                app.borrow_mut().start(minutes, Rc::downgrade(&app));
            }
        }
    });
    app.borrow().tray.ui.on_abort({
        let app = Rc::downgrade(&app);
        move || {
            if let Some(app) = app.upgrade() {
                app.borrow_mut().abort();
            }
        }
    });
    app.borrow().tray.ui.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
    app.borrow().tray.ui.on_icon_clicked({
        let tray = app.borrow().tray.ui.as_weak();
        let previous_click = Cell::new(None::<Instant>);
        move || {
            if let Some(tray) = tray.upgrade() {
                let now = Instant::now();
                if previous_click.replace(Some(now)).is_some_and(|previous| {
                    now.duration_since(previous) <= Duration::from_millis(500)
                }) {
                    previous_click.set(None);
                    tray.invoke_restart();
                } else {
                    tray.invoke_settings();
                }
            }
        }
    });
    if !desktop::tray_available() {
        ensure_settings_window(&settings_window, &about_window, &app, &directory)?;
        settings_window
            .borrow()
            .as_ref()
            .expect("Einstellungen-Fenster wurde soeben erzeugt")
            .set_standalone(true);
        app.borrow().tray.ui.hide()?;
        settings_window
            .borrow()
            .as_ref()
            .expect("Einstellungen-Fenster wurde soeben erzeugt")
            .show()?;
    }
    app.borrow().sync_window();
    app.borrow().tray.notify("Timer gestartet", false);
    slint::run_event_loop_until_quit()?;
    app.borrow().tick.stop();
    Ok(())
}
