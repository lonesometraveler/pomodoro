use crate::display;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use serde::{Deserialize, Serialize};

/// Signal for speaker
pub static SPEAKER_SIGNAL: Signal<ThreadModeRawMutex, Buzzer> = Signal::new();
/// Signal for setting
pub static CONFIG_SIGNAL: Signal<ThreadModeRawMutex, heapless::Vec<u8, 32>> = Signal::new();

/// Buzzer
pub enum Buzzer {
    Start,
    Error,
}

impl Buzzer {
    pub fn freq(&self) -> u32 {
        match self {
            Self::Start => 880,
            Self::Error => 185,
        }
    }

    pub fn duty(&self) -> u16 {
        16383
    }

    pub fn duration_ms(&self) -> u64 {
        match self {
            Self::Start => 200,
            Self::Error => 500,
        }
    }
}

/// App's state
#[derive(Copy, Clone, Debug, defmt::Format)]
pub enum State {
    Paused,
    Running,
    Resting,
}

impl State {
    pub fn toggle(self) -> State {
        use State::*;
        match self {
            Paused => Running,
            Running => Paused,
            Resting => Resting,
        }
    }

    pub fn next(self) -> State {
        use State::*;
        match self {
            Paused => Paused,
            Running => Resting,
            Resting => Running,
        }
    }

    pub fn bitmap(&self) -> display::Frame<5, 5> {
        match self {
            State::Paused => display::bitmaps::CROSS_MARK,
            State::Running => display::bitmaps::ARROW_RIGHT,
            State::Resting => display::bitmaps::CHECK_MARK,
        }
    }
}

/// Timer Configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct TimerConfig {
    /// Work time in seconds
    pub work_time: u32,
    /// Rest time in seconds
    pub rest_time: u32,
}

impl Default for TimerConfig {
    fn default() -> Self {
        TimerConfig {
            work_time: 25 * 60,
            rest_time: 5 * 60,
        }
    }
}

impl defmt::Format for TimerConfig {
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(
            f,
            "work: {} mins / rest: {} mins",
            self.work_time / 60,
            self.rest_time / 60
        );
    }
}

impl TimerConfig {
    /// Returns a timer for a specified `State`
    pub fn timer_for(&self, state: State) -> u32 {
        use State::*;
        match state {
            Paused => 0,
            Running => self.work_time,
            Resting => self.rest_time,
        }
    }
}

/// Possible Errors from configuration service
#[derive(defmt::Format)]
pub enum ConfigError {
    InvalidFormat(heapless::Vec<u8, 32>),
    Utf8(#[defmt(Debug2Format)] core::str::Utf8Error),
    Parse(#[defmt(Debug2Format)] core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for ConfigError {
    fn from(e: core::num::ParseIntError) -> Self {
        ConfigError::Parse(e)
    }
}

impl From<core::str::Utf8Error> for ConfigError {
    fn from(e: core::str::Utf8Error) -> Self {
        ConfigError::Utf8(e)
    }
}
