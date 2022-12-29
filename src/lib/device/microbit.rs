//! Board definition for Microbit v2
use crate::display::LedMatrix as LedMatrixDriver;
use embassy_nrf::{
    gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull},
    interrupt::{self, InterruptExt, Priority},
    peripherals::{PWM0, TWISPI0, UARTE0},
    pwm::{Prescaler, SimplePwm},
    twim::Twim,
    uarte::{self, Uarte},
};

/// LED matrix peripheral for the micro:bit
pub type LedMatrix = LedMatrixDriver<Output<'static, AnyPin>, 5, 5>;

pub struct Board {
    /// Onboard Button A
    pub button1: Input<'static, AnyPin>,
    /// Onboard Button B
    pub button2: Input<'static, AnyPin>,
    /// TWI
    pub twim: Twim<'static, TWISPI0>,
    /// UART: Serial - USB
    pub uart: Uarte<'static, UARTE0>,
    /// LED display
    pub display: LedMatrix,
    /// PWM pin
    pub pwm: SimplePwm<'static, PWM0>,
}

impl Board {
    /// Returns Board with concrete peripherals
    pub fn init(p: embassy_nrf::Peripherals) -> Board {
        // 5x5 LED display
        let rows = [
            output_pin(p.P0_21.degrade()),
            output_pin(p.P0_22.degrade()),
            output_pin(p.P0_15.degrade()),
            output_pin(p.P0_24.degrade()),
            output_pin(p.P0_19.degrade()),
        ];

        let cols = [
            output_pin(p.P0_28.degrade()),
            output_pin(p.P0_11.degrade()),
            output_pin(p.P0_31.degrade()),
            output_pin(p.P1_05.degrade()),
            output_pin(p.P0_30.degrade()),
        ];
        let display = LedMatrixDriver::new(rows, cols);

        // Buttons
        let button1 = Input::new(p.P0_14.degrade(), Pull::Up);
        let button2 = Input::new(p.P0_23.degrade(), Pull::Up);

        // TWI
        let twim_config = embassy_nrf::twim::Config::default();
        let twim_irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
        twim_irq.set_priority(Priority::P3);
        let twim = Twim::new(p.TWISPI0, twim_irq, p.P0_16, p.P0_08, twim_config);

        // UART
        let mut uart_config = uarte::Config::default();
        uart_config.parity = uarte::Parity::EXCLUDED;
        uart_config.baudrate = uarte::Baudrate::BAUD115200;
        let uart_irq = interrupt::take!(UARTE0_UART0);
        uart_irq.set_priority(Priority::P3);
        let uart = uarte::Uarte::new(p.UARTE0, uart_irq, p.P1_08, p.P0_06, uart_config);

        // PWM
        let pwm = embassy_nrf::pwm::SimplePwm::new_1ch(p.PWM0, p.P0_00);
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(32767);

        Board {
            display,
            button1,
            button2,
            twim,
            uart,
            pwm,
        }
    }
}

fn output_pin(pin: AnyPin) -> Output<'static, AnyPin> {
    Output::new(pin, Level::Low, OutputDrive::Standard)
}
