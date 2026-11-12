use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    #[default]
    Pomodoro,
    Stopwatch,
    Countdown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub focus: u32,
    pub short_break: u32,
    pub long_break: u32,
    pub cycles: u32,
    pub countdown: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { focus: 1500, short_break: 300, long_break: 900, cycles: 4, countdown: 900 }
    }
}

impl Settings {
    pub fn valid(&self) -> bool {
        [self.focus, self.short_break, self.long_break, self.countdown]
            .iter()
            .all(|value| (1..=359_999).contains(value))
            && (1..=20).contains(&self.cycles)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub mode: Mode,
    pub seconds: u64,
    pub running: bool,
    pub elapsed: bool,
    pub phase: &'static str,
    pub cycle: u32,
    pub alarm: u32,
    pub notice: Option<&'static str>,
    pub laps: Vec<u64>,
    pub settings: Settings,
}

#[derive(Default)]
struct Clock {
    accumulated: Duration,
    start: Option<Instant>,
}

impl Clock {
    fn elapsed(&self, now: Instant) -> Duration {
        self.accumulated + self.start.map_or(Duration::ZERO, |start| now - start)
    }

    fn pause(&mut self, now: Instant) {
        self.accumulated = self.elapsed(now);
        self.start = None;
    }
}

#[derive(Default)]
pub struct Timer {
    clocks: [Clock; 3],
    mode: Mode,
    settings: Settings,
    completed_focus: u32,
    on_break: bool,
    alarm: u32,
    notice: Option<&'static str>,
    laps: Vec<u64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Action {
    Toggle,
    Reset,
    Select(Mode),
    Lap,
    Configure(Settings),
}

impl Timer {
    pub fn new(settings: Settings) -> Self {
        Self { settings, ..Self::default() }
    }

    fn slot(&self) -> usize {
        match self.mode {
            Mode::Pomodoro => 0,
            Mode::Stopwatch => 1,
            Mode::Countdown => 2,
        }
    }

    fn long_break(&self) -> bool {
        self.on_break && self.completed_focus % self.settings.cycles == 0
    }

    fn limit(&self) -> Option<Duration> {
        let seconds = match self.mode {
            Mode::Stopwatch => return None,
            Mode::Countdown => self.settings.countdown,
            Mode::Pomodoro if self.long_break() => self.settings.long_break,
            Mode::Pomodoro if self.on_break => self.settings.short_break,
            Mode::Pomodoro => self.settings.focus,
        };
        Some(Duration::from_secs(u64::from(seconds)))
    }

    pub fn tick(&mut self, now: Instant) {
        let slot = self.slot();
        if let Some(limit) = self.limit() {
            if self.clocks[slot].start.is_some() && self.clocks[slot].elapsed(now) >= limit {
                self.alarm = self.alarm.wrapping_add(1);
                if self.mode == Mode::Pomodoro {
                    self.on_break = !self.on_break;
                    if self.on_break {
                        self.completed_focus = self.completed_focus.saturating_add(1);
                    }
                    self.clocks[slot] = Clock::default();
                    self.notice = Some(if self.on_break { "Foco concluído" } else { "Pausa concluída" });
                } else {
                    self.clocks[slot] = Clock { accumulated: limit, start: None };
                    self.notice = Some("Tempo encerrado");
                }
            }
        }
    }

    pub fn apply(&mut self, action: Action, now: Instant) -> Result<(), String> {
        self.tick(now);
        let slot = self.slot();
        match action {
            Action::Toggle => {
                if self.clocks[slot].start.is_some() {
                    self.clocks[slot].pause(now);
                } else {
                    if self.limit().is_some_and(|limit| self.clocks[slot].elapsed(now) >= limit) {
                        self.clocks[slot] = Clock::default();
                    }
                    self.clocks[slot].start = Some(now);
                }
                self.notice = None;
            }
            Action::Reset => {
                self.clocks[slot] = Clock::default();
                if self.mode == Mode::Stopwatch {
                    self.laps.clear();
                }
                self.notice = None;
            }
            Action::Select(mode) => {
                if self.mode != mode {
                    self.clocks[slot].pause(now);
                    self.mode = mode;
                    self.notice = None;
                }
            }
            Action::Lap => {
                if self.mode == Mode::Stopwatch && self.clocks[slot].start.is_some() {
                    self.laps.push(self.clocks[slot].elapsed(now).as_millis() as u64);
                }
            }
            Action::Configure(settings) => {
                if !settings.valid() {
                    return Err("Informe durações de 1 segundo a 99:59:59 e de 1 a 20 blocos.".into());
                }
                if self.settings.countdown != settings.countdown {
                    self.clocks[2] = Clock::default();
                }
                if self.settings.focus != settings.focus
                    || self.settings.short_break != settings.short_break
                    || self.settings.long_break != settings.long_break
                    || self.settings.cycles != settings.cycles
                {
                    self.clocks[0] = Clock::default();
                    self.completed_focus = 0;
                    self.on_break = false;
                }
                self.settings = settings;
                self.notice = None;
            }
        }
        Ok(())
    }

    pub fn snapshot(&self, now: Instant) -> Snapshot {
        let clock = &self.clocks[self.slot()];
        let elapsed = clock.elapsed(now);
        let seconds = self.limit().map_or(elapsed.as_secs(), |limit| {
            let remaining = limit.saturating_sub(elapsed);
            remaining.as_secs() + if remaining.subsec_nanos() > 0 { 1 } else { 0 }
        });
        let phase = match self.mode {
            Mode::Stopwatch => "Cronômetro",
            Mode::Countdown => "Temporizador",
            Mode::Pomodoro if self.long_break() => "Pausa longa",
            Mode::Pomodoro if self.on_break => "Pausa curta",
            Mode::Pomodoro => "Foco",
        };
        Snapshot {
            mode: self.mode,
            seconds,
            running: clock.start.is_some(),
            elapsed: elapsed > Duration::ZERO,
            phase,
            cycle: if self.on_break { self.completed_focus } else { self.completed_focus.saturating_add(1) },
            alarm: self.alarm,
            notice: self.notice,
            laps: self.laps.clone(),
            settings: self.settings.clone(),
        }
    }
}
