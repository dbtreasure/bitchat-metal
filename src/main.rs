#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Main entry point for bitchat-metal firmware
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("bitchat-metal v0.1.0 starting...");

    // Initialize Embassy for nRF52840
    let p = embassy_nrf::init(Default::default());

    // LED for status indication (P0.13 = LED1 on nRF52840 DK)
    let mut status_led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    defmt::info!("Hardware initialized. Starting main loop...");

    // Heartbeat to show firmware is running
    // TODO: Replace with BLE initialization in M1
    loop {
        status_led.set_high();
        Timer::after_millis(100).await;
        status_led.set_low();
        Timer::after_millis(1900).await;
    }
}