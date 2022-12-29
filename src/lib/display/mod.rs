//! Driver a NxM LED matrix display
//!
//! * Can display 5x5 bitmaps from raw data or characters
//! * Methods for scrolling text across LED matrix or displaying a bitmap for a duration
use embassy_time::{block_for, Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

pub mod bitmaps;

mod types;
pub use types::*;

const REFRESH_INTERVAL: Duration = Duration::from_micros(500);

/// Led matrix driver supporting arbitrary sized led matrixes.
///
/// NOTE: Currently restricted by 8 bits width
pub struct LedMatrix<P, const ROWS: usize, const COLS: usize>
where
    P: OutputPin + 'static,
{
    pin_rows: [P; ROWS],
    pin_cols: [P; COLS],
    frame_buffer: Frame<COLS, ROWS>,
    row_p: usize,
    brightness: Brightness,
}

impl<P, const ROWS: usize, const COLS: usize> LedMatrix<P, ROWS, COLS>
where
    P: OutputPin,
{
    /// Create a new instance of an LED matrix using the provided pins
    pub fn new(pin_rows: [P; ROWS], pin_cols: [P; COLS]) -> Self {
        LedMatrix {
            pin_rows,
            pin_cols,
            frame_buffer: Frame::empty(),
            row_p: 0,
            brightness: Default::default(),
        }
    }

    /// Clear all LEDs
    pub fn clear(&mut self) {
        self.frame_buffer.clear();
        for row in self.pin_rows.iter_mut() {
            row.set_high().ok();
        }

        for col in self.pin_cols.iter_mut() {
            col.set_high().ok();
        }
    }

    /// Turn on point (x,y) in the frame buffer
    pub fn on(&mut self, x: usize, y: usize) {
        self.frame_buffer.set(x, y);
    }

    /// Turn off point (x,y) in the frame buffer
    pub fn off(&mut self, x: usize, y: usize) {
        self.frame_buffer.unset(x, y);
    }

    /// Apply the provided frame onto the frame buffer
    pub fn apply(&mut self, frame: Frame<COLS, ROWS>) {
        self.frame_buffer = frame;
    }

    /// Adjust the brightness level
    pub fn set_brightness(&mut self, brightness: Brightness) {
        self.brightness = brightness;
    }

    /// Increase brightness relative to current setting
    pub fn increase_brightness(&mut self) {
        self.brightness += 1;
    }

    /// Decrease brightness relative to current setting
    pub fn decrease_brightness(&mut self) {
        self.brightness -= 1;
    }

    /// Perform a full refresh of the display based on the current frame buffer
    pub fn render(&mut self) {
        for row in self.pin_rows.iter_mut() {
            row.set_low().ok();
        }

        for (cid, col) in self.pin_cols.iter_mut().enumerate() {
            if self.frame_buffer.is_set(cid, self.row_p) {
                col.set_low().ok();
            } else {
                col.set_high().ok();
            }
        }

        // Adjust interval will impact brightness of the LEDs
        block_for(Duration::from_micros(
            ((Brightness::MAX.level() - self.brightness.level()) as u64) * 6000
                / Brightness::MAX.level() as u64,
        ));

        self.pin_rows[self.row_p].set_high().ok();

        self.row_p = (self.row_p + 1) % self.pin_rows.len();
    }

    /// Display the provided frame for the duration. Handles screen refresh
    /// in an async display loop.
    pub async fn display(&mut self, frame: Frame<COLS, ROWS>, length: Duration) {
        self.apply(frame);
        let end = Instant::now() + length;
        while Instant::now() < end {
            self.render();
            Timer::after(REFRESH_INTERVAL).await;
        }
        self.clear();
    }
}
