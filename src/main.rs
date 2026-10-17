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
    window: slint::Weak<SettingsWindow>,
}

impl App {
    fn sync_window(&self) {
        if let Some(window) = self.window.upgrade() {
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
            if let Some(window) = self.window.upgrade() {
                window.set_timer_status("Timer abgelaufen!".into());
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
        if let Some(window) = self.window.upgrade() {
            let path = window.get_sound_file();
            if window.get_sound_enabled() && self.check_sound(&path) {
                desktop::play_sound(Path::new(path.as_str()));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = SettingsWindow::new()?;
    let executable = std::env::current_exe()?;
    let directory: PathBuf = executable.parent().unwrap().into();
    window.set_sound_file(directory.join("ding.wav").to_string_lossy().as_ref().into());
    window.window().on_close_requested({
        let window = window.as_weak();
        move || {
            if window
                .upgrade()
                .is_some_and(|window| window.get_standalone())
            {
                let _ = slint::quit_event_loop();
            }
            slint::CloseRequestResponse::HideWindow
        }
    });
    let about = AboutWindow::new()?;
    about.on_dismiss({
        let about = about.as_weak();
        move || {
            if let Some(about) = about.upgrade() {
                let _ = about.hide();
            }
        }
    });
    let app = Rc::new(RefCell::new(App {
        countdown: Countdown::default(),
        tray: Tray::new()?,
        tick: slint::Timer::default(),
        window: window.as_weak(),
    }));

    window.on_quit(|| {
        let _ = slint::quit_event_loop();
    });
    window.on_about({
        let about = about.as_weak();
        move || {
            if let Some(about) = about.upgrade() {
                let _ = about.show();
            }
        }
    });
    window.on_start({
        let app = app.clone();
        move |minutes| app.borrow_mut().start(minutes as u32, Rc::downgrade(&app))
    });
    window.on_restart({
        let app = app.clone();
        move || {
            let minutes = app.borrow().countdown.last_minutes;
            app.borrow_mut().start(minutes, Rc::downgrade(&app));
        }
    });
    window.on_abort({
        let app = app.clone();
        move || app.borrow_mut().abort()
    });
    window.on_play({
        let app = app.clone();
        move || app.borrow().play()
    });
    window.on_accept({
        let app = app.clone();
        let window = window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                if app.borrow().check_sound(&window.get_sound_file()) && !window.get_standalone() {
                    let _ = window.hide();
                }
            }
        }
    });
    window.on_browse({
        let window = window.as_weak();
        move || {
            let directory = directory.clone();
            let window = window.clone();
            let _ = slint::spawn_local(async move {
                let selected = desktop::choose_sound(&directory).await;
                if let (Some(path), Some(window)) = (selected, window.upgrade()) {
                    window.set_sound_file(path.to_string_lossy().as_ref().into());
                }
            });
        }
    });
    window.on_duration_changed({
        let app = app.clone();
        move |minutes| app.borrow().tray.set_custom_minutes(minutes as u32)
    });

    app.borrow().tray.ui.on_settings({
        let window = window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                if let Err(error) = window.show() {
                    eprintln!(
                        "Timer: Einstellungen-Fenster konnte nicht angezeigt werden: {error}"
                    );
                }
                window.window().request_redraw();
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
        window.set_standalone(true);
        app.borrow().tray.ui.hide()?;
        window.show()?;
    }
    app.borrow().sync_window();
    app.borrow().tray.notify("Timer gestartet", false);
    slint::run_event_loop_until_quit()?;
    app.borrow().tick.stop();
    Ok(())
}
