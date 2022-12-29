#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, select3, Either, Either3};
use embassy_nrf::{interrupt::Priority, pwm::SimplePwm};
use embassy_time::{Duration, Timer};
use microbit_pomodoro::{
    self as _,
    ble::{sd, server},
    device::Board,
    display,
    flash_storage::FlashStorage,
    types::{Buzzer, ConfigError, State, TimerConfig, CONFIG_SIGNAL, SPEAKER_SIGNAL},
};
use nrf_softdevice::{Flash, Softdevice};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Configure peripherals
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);
    let board = Board::init(p);

    // Enable SoftDevice
    let sd = nrf_softdevice::Softdevice::enable(&sd::softdevice_config());
    // Create BLE GATT server
    let server = unwrap!(server::Server::new(sd));
    // Run SoftDevice task
    unwrap!(spawner.spawn(sd::softdevice_task(sd)));
    // Run BLE server task
    unwrap!(spawner.spawn(server::ble_server_task(server, sd)));

    // Run Speaker task
    let speaker = board.pwm;
    unwrap!(spawner.spawn(speak(speaker)));

    // Button B: Start button
    let start_button = board.button2;

    // Load timer config
    let timer_config = load_timer_config(sd, start_button).await;
    info!("config: {:?}", timer_config);

    // Init State
    let mut app_state = State::Running;

    // Init pomodoro timer
    let mut pomo_timer = timer_config.timer_for(app_state);

    // Set up display
    let mut display = board.display;
    display.set_brightness(display::Brightness::MAX);

    SPEAKER_SIGNAL.signal(Buzzer::Start);

    // Button A: Play/Pause button
    let mut play_pause_button = board.button1;

    loop {
        // Load a bitmap for the app state
        let bitmap = app_state.bitmap();

        let timer_future = Timer::after(Duration::from_secs(1));
        let button_future = play_pause_button.wait_for_falling_edge();
        let display_future = display.display(bitmap, Duration::from_millis(1500));

        match select3(timer_future, button_future, display_future).await {
            // Timer expired
            Either3::First(_) => match app_state {
                State::Running | State::Resting => {
                    pomo_timer -= 1;
                    if pomo_timer == 0 {
                        app_state = app_state.next();
                        pomo_timer = timer_config.timer_for(app_state);
                        SPEAKER_SIGNAL.signal(Buzzer::Start);
                    }
                }
                _ => (),
            },
            // Button pressed
            Either3::Second(_) => {
                app_state = app_state.toggle();
            }
            // Display timer expired (This never happens as timer_future or button_future always runs to completion first.)
            Either3::Third(_) => {}
        };
    }
}

/// Speaker task
#[embassy_executor::task]
async fn speak(mut speaker: SimplePwm<'static, embassy_nrf::peripherals::PWM0>) {
    loop {
        let buzz = SPEAKER_SIGNAL.wait().await;
        speaker.set_period(buzz.freq());
        speaker.set_duty(0, buzz.duty());
        Timer::after(Duration::from_millis(buzz.duration_ms())).await;
        speaker.set_duty(0, 0);
    }
}

/// Load a timer
async fn load_timer_config(
    sd: &Softdevice,
    mut start_button: embassy_nrf::gpio::Input<'_, embassy_nrf::gpio::AnyPin>,
) -> TimerConfig {
    // Set up Flash Storage
    let mut f = Flash::take(sd);
    const FS_START_ADDR: u32 = 0x7F000;
    const FS_END_ADDR: u32 = 0x80000;
    let f_storage = FlashStorage::new(FS_START_ADDR, FS_END_ADDR);

    const BUFF_SIZE: usize = 16;
    let mut buf = [0u8; BUFF_SIZE];

    // Configure a new timer via BLE or hit Button B to start a pre-configured timer.
    loop {
        let button_future = start_button.wait_for_falling_edge();
        let config_future = wait_for_new_config();

        match select(button_future, config_future).await {
            // Button pressed
            Either::First(_) => break,
            // Config request received from BLE
            Either::Second(result) => match result {
                // Valid request
                Ok(config) => {
                    if postcard::to_slice_cobs(&config, &mut buf).is_ok()
                        && f_storage.write(&mut f, &mut buf).await.is_ok()
                    {
                        break;
                    }
                }
                // Invalid request
                Err(e) => {
                    error!("{:?}", e);
                    SPEAKER_SIGNAL.signal(Buzzer::Error);
                }
            },
        }
    }

    // Return a pre-defined timer if it exits. Otherwise, return a default timer.
    if f_storage.read(&mut f, &mut buf).await.is_ok() {
        match postcard::from_bytes_cobs(&mut buf) {
            Ok(data) => data,
            Err(_) => TimerConfig::default(),
        }
    } else {
        TimerConfig::default()
    }
}

/// Waits for a signal from BLE service
async fn wait_for_new_config() -> Result<TimerConfig, ConfigError> {
    let data = CONFIG_SIGNAL.wait().await;
    let mut timers = data.split(|b| b == &b',');

    if let (Some(w), Some(r)) = (timers.next(), timers.next()) {
        Ok(TimerConfig {
            work_time: core::str::from_utf8(w)?.parse::<u32>()? * 60,
            rest_time: core::str::from_utf8(r)?.parse::<u32>()? * 60,
        })
    } else {
        Err(ConfigError::InvalidFormat(data))
    }
}
