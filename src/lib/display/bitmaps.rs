use super::*;

#[rustfmt::skip]
    /// A check-mark bitmap
    pub const CHECK_MARK: Frame<5, 5> = frame_5x5(&[
        0b00000,
        0b00001,
        0b00010,
        0b10100,
        0b01000,
    ]);

#[rustfmt::skip]
    /// A cross-mark bitmap
    pub const CROSS_MARK: Frame<5, 5> = frame_5x5(&[
        0b00000,
        0b01010,
        0b00100,
        0b01010,
        0b00000,
    ]);

#[rustfmt::skip]
    /// A left arrow bitmap
    pub const ARROW_LEFT: Frame<5, 5> = frame_5x5(&[
        0b00100,
        0b01000,
        0b11111,
        0b01000,
        0b00100,
    ]);

#[rustfmt::skip]
    /// A right arrow bitmap
    pub const ARROW_RIGHT: Frame<5, 5> = frame_5x5(&[
        0b00100,
        0b00010,
        0b11111,
        0b00010,
        0b00100,
    ]);

/// Construct a 5x5 frame from a byte slice
pub const fn frame_5x5<const XSIZE: usize, const YSIZE: usize>(
    input: &[u8; 5],
) -> Frame<XSIZE, YSIZE> {
    let mut data = [Bitmap::empty(5); YSIZE];
    data[0] = Bitmap::new(input[0], 5);
    data[1] = Bitmap::new(input[1], 5);
    data[2] = Bitmap::new(input[2], 5);
    data[3] = Bitmap::new(input[3], 5);
    data[4] = Bitmap::new(input[4], 5);
    Frame::new(data)
}
