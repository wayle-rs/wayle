use std::sync::{Arc, Mutex};

use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PomodoroMode {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PomodoroSnapshot {
    pub(crate) mode: PomodoroMode,
    pub(crate) timer_state: TimerState,
    pub(crate) remaining_seconds: u32,
}

impl PomodoroSnapshot {
    pub(crate) fn format_time(self) -> String {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{minutes:02}:{seconds:02}")
    }
}

struct PomodoroTimerState {
    mode: PomodoroMode,
    timer_state: TimerState,
    remaining_seconds: u32,
    work_duration: u32,
    short_break_duration: u32,
    long_break_duration: u32,
    cycles_before_long_break: u32,
    completed_cycles: u32,
}

impl PomodoroTimerState {
    fn new(
        work_duration: u32,
        short_break_duration: u32,
        long_break_duration: u32,
        cycles_before_long_break: u32,
    ) -> Self {
        Self {
            mode: PomodoroMode::Work,
            timer_state: TimerState::Stopped,
            remaining_seconds: work_duration * 60,
            work_duration,
            short_break_duration,
            long_break_duration,
            cycles_before_long_break,
            completed_cycles: 0,
        }
    }

    fn snapshot(&self) -> PomodoroSnapshot {
        PomodoroSnapshot {
            mode: self.mode,
            timer_state: self.timer_state,
            remaining_seconds: self.remaining_seconds,
        }
    }

    fn tick(&mut self) -> bool {
        if self.timer_state != TimerState::Running {
            return false;
        }

        if self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
        } else {
            self.on_timer_complete();
        }
        true
    }

    fn start(&mut self) {
        self.timer_state = TimerState::Running;
    }

    fn pause(&mut self) {
        self.timer_state = TimerState::Paused;
    }

    fn switch_mode(&mut self, mode: PomodoroMode) {
        self.mode = mode;
        self.timer_state = TimerState::Stopped;
        self.remaining_seconds = match mode {
            PomodoroMode::Work => self.work_duration * 60,
            PomodoroMode::ShortBreak => self.short_break_duration * 60,
            PomodoroMode::LongBreak => self.long_break_duration * 60,
        };
    }

    fn update_durations(&mut self, work: u32, short_break: u32, long_break: u32, cycles: u32) {
        self.work_duration = work;
        self.short_break_duration = short_break;
        self.long_break_duration = long_break;
        self.cycles_before_long_break = cycles;

        if self.timer_state == TimerState::Stopped {
            self.remaining_seconds = match self.mode {
                PomodoroMode::Work => work * 60,
                PomodoroMode::ShortBreak => short_break * 60,
                PomodoroMode::LongBreak => long_break * 60,
            };
        }
    }

    fn on_timer_complete(&mut self) {
        match self.mode {
            PomodoroMode::Work => {
                self.completed_cycles += 1;
                if self.completed_cycles >= self.cycles_before_long_break {
                    self.switch_mode(PomodoroMode::LongBreak);
                    self.completed_cycles = 0;
                } else {
                    self.switch_mode(PomodoroMode::ShortBreak);
                }
            }
            PomodoroMode::ShortBreak | PomodoroMode::LongBreak => {
                self.switch_mode(PomodoroMode::Work);
            }
        }
    }
}

pub(crate) struct PomodoroSharedState {
    state: Mutex<PomodoroTimerState>,
    sender: watch::Sender<PomodoroSnapshot>,
}

impl PomodoroSharedState {
    pub(crate) fn new(
        work_duration: u32,
        short_break_duration: u32,
        long_break_duration: u32,
        cycles_before_long_break: u32,
    ) -> Self {
        let state = PomodoroTimerState::new(
            work_duration,
            short_break_duration,
            long_break_duration,
            cycles_before_long_break,
        );
        let snapshot = state.snapshot();
        let (sender, _) = watch::channel(snapshot);
        Self {
            state: Mutex::new(state),
            sender,
        }
    }

    pub(crate) fn snapshot(&self) -> PomodoroSnapshot {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .snapshot()
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<PomodoroSnapshot> {
        self.sender.subscribe()
    }

    pub(crate) fn tick(&self) -> Option<PomodoroSnapshot> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !state.tick() {
            return None;
        }

        let snapshot = state.snapshot();
        let _ = self.sender.send(snapshot);
        Some(snapshot)
    }

    pub(crate) fn start(&self) -> PomodoroSnapshot {
        self.mutate(|state| state.start())
    }

    pub(crate) fn pause(&self) -> PomodoroSnapshot {
        self.mutate(|state| state.pause())
    }

    pub(crate) fn switch_mode(&self, mode: PomodoroMode) -> PomodoroSnapshot {
        self.mutate(|state| state.switch_mode(mode))
    }

    pub(crate) fn update_durations(
        &self,
        work: u32,
        short_break: u32,
        long_break: u32,
        cycles: u32,
    ) -> PomodoroSnapshot {
        self.mutate(|state| state.update_durations(work, short_break, long_break, cycles))
    }

    fn mutate(&self, f: impl FnOnce(&mut PomodoroTimerState)) -> PomodoroSnapshot {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        f(&mut state);
        let snapshot = state.snapshot();
        let _ = self.sender.send(snapshot);
        snapshot
    }
}

pub(crate) type SharedPomodoroState = Arc<PomodoroSharedState>;
