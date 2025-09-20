#![no_std]
#![no_main]

mod ble;
mod config;
mod protocol;
mod bitchat;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use ble::service::BitchatServer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("bitchat-metal v0.1.0 starting...");

    // Configure Embassy with softdevice-compatible interrupt priorities
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
    config.time_interrupt_priority = embassy_nrf::interrupt::Priority::P2;

    let p = embassy_nrf::init(config);

    let mut status_led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    let mut connect_led = Output::new(p.P0_14, Level::Low, OutputDrive::Standard);

    info!("Hardware initialized. Initializing BLE stack...");

    // Quick LED test to show we're starting
    for _ in 0..3 {
        status_led.set_high();
        connect_led.set_high();
        Timer::after_millis(200).await;
        status_led.set_low();
        connect_led.set_low();
        Timer::after_millis(200).await;
    }

    let sd = ble::init(&spawner);

    info!("Softdevice enabled, spawning BLE task...");
    spawner.must_spawn(ble_task(sd, connect_led));

    info!("BLE task spawned. Starting heartbeat...");

    loop {
        status_led.set_high();
        Timer::after_millis(100).await;
        status_led.set_low();
        Timer::after_millis(1900).await;
    }
}

#[embassy_executor::task]
async fn ble_task(sd: &'static mut nrf_softdevice::Softdevice, mut connect_led: Output<'static>) {
    let mut server = match BitchatServer::new(sd) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to create GATT server: {:?}", e);
            return;
        }
    };

    info!("GATT server created. Starting advertisement loop...");

    loop {
        connect_led.set_low();

        let conn = match ble::advertise::advertise(sd).await {
            Ok(conn) => {
                info!("Connection established!");
                connect_led.set_high();
                conn
            }
            Err(e) => {
                warn!("Advertisement error: {:?}", e);
                Timer::after_millis(1000).await;
                continue;
            }
        };

        // Run server (removed timeout for now since run() doesn't return Result)
        server.run(&conn).await;

        info!("Connection lost or timed out, restarting advertisement");
    }
}