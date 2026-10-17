#[derive(Default)]
pub struct Countdown {
    pub last_minutes: u32,
    pub seconds_remaining: u32,
    pub running: bool,
}

impl Countdown {
    pub fn start(&mut self, minutes: u32) -> bool {
        if minutes == 0 {
            return false;
        }
        self.last_minutes = minutes;
        self.seconds_remaining = minutes * 60;
        self.running = true;
        true
    }

    pub fn abort(&mut self) {
        self.running = false;
        self.seconds_remaining = 0;
    }

    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }
        self.seconds_remaining = self.seconds_remaining.saturating_sub(1);
        if self.seconds_remaining == 0 {
            self.running = false;
            return true;
        }
        false
    }

    pub fn tooltip(&self) -> String {
        if self.running {
            format!(
                "Timer läuft! (noch {}:{:02} Minuten)\n\nLinksklick: Einstellungen",
                self.seconds_remaining / 60,
                self.seconds_remaining % 60
            )
        } else {
            let mut text = String::from(
                "Timer\n\nLinksklick: Einstellungen\nRechtsklick: Funktionen wählen\nDoppelklick: Timer neu starten",
            );
            if self.last_minutes > 0 {
                text.push_str(&format!(" ({})", minutes_text(self.last_minutes)));
            }
            text
        }
    }
}

pub fn minutes_text(minutes: u32) -> String {
    format!("{} Minute{}", minutes, if minutes == 1 { "" } else { "n" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_double_click_requires_a_duration() {
        let mut timer = Countdown::default();
        assert!(!timer.start(timer.last_minutes));
        assert!(!timer.running);
        assert!(!timer.tick());
    }

    #[test]
    fn expires_exactly_once_after_sixty_ticks() {
        let mut timer = Countdown::default();
        assert!(timer.start(1));
        for _ in 0..59 {
            assert!(!timer.tick());
        }
        assert_eq!(timer.seconds_remaining, 1);
        assert!(timer.tick());
        assert!(!timer.running);
        assert!(!timer.tick());
        assert_eq!(timer.last_minutes, 1);
    }

    #[test]
    fn double_click_restarts_a_running_timer() {
        let mut timer = Countdown::default();
        timer.start(3);
        timer.tick();
        assert!(timer.start(timer.last_minutes));
        assert_eq!(timer.seconds_remaining, 180);
    }

    #[test]
    fn abort_preserves_the_last_duration() {
        let mut timer = Countdown::default();
        timer.start(11);
        timer.abort();
        assert_eq!(timer.seconds_remaining, 0);
        assert!(!timer.running);
        assert!(!timer.tick());
        assert!(timer.start(timer.last_minutes));
        assert_eq!(timer.seconds_remaining, 660);
    }

    #[test]
    fn labels_match_the_original() {
        assert_eq!(minutes_text(1), "1 Minute");
        assert_eq!(minutes_text(11), "11 Minuten");
        let mut timer = Countdown::default();
        assert!(timer.tooltip().contains("Linksklick: Einstellungen"));
        timer.start(2);
        timer.tick();
        assert_eq!(
            timer.tooltip(),
            "Timer läuft! (noch 1:59 Minuten)\n\nLinksklick: Einstellungen"
        );
        timer.abort();
        assert!(timer.tooltip().contains("Linksklick: Einstellungen"));
        assert!(timer.tooltip().ends_with("(2 Minuten)"));
    }
}
