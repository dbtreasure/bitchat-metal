#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("TEST: bitchat-metal hardware test starting...");

    let p = embassy_nrf::init(Default::default());

    let mut led1 = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    info!("TEST: GPIO initialized, starting simple blink test...");
    info!("TEST: If LED1 blinks with pattern, firmware is working!");

    // Simple pattern: 3 fast blinks, then pause
    loop {
        // 3 fast blinks
        for _ in 0..3 {
            led1.set_high();
            Timer::after_millis(100).await;
            led1.set_low();
            Timer::after_millis(100).await;
        }

        // Pause
        Timer::after_millis(1000).await;

        info!("TEST: Blink cycle complete");
    }
}