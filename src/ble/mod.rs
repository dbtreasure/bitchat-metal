pub mod advertise;
pub mod service;

use embassy_executor::Spawner;
use nrf_softdevice::{raw, Softdevice};


pub fn init(spawner: &Spawner) -> &'static mut Softdevice {
    use defmt::info;

    info!("Starting softdevice configuration...");

    // Simplified config - let defaults handle most settings
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_250_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t {
            att_mtu: 247,
        }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 1024,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        ..Default::default()
    };

    info!("Enabling softdevice...");
    let sd = Softdevice::enable(&config);
    info!("Softdevice enabled successfully!");

    unsafe {
        let ptr = sd as *const _ as *const Softdevice;
        info!("Spawning softdevice task...");
        spawner.must_spawn(softdevice_task(&*ptr));
        let mutable_ptr = sd as *const _ as *mut Softdevice;
        info!("Softdevice initialization complete");
        &mut *mutable_ptr
    }
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}