//! BLE Server Config and Tasks
use super::services::*;
use crate::types::CONFIG_SIGNAL;
use defmt::*;
use nrf_softdevice::{
    ble::{gatt_server, peripheral},
    Softdevice,
};

/// BLE advertising data
#[rustfmt::skip]
const ADV_DATA: &[u8; 18] =
    &[
        0x02, 0x01, nrf_softdevice::raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'P', b'o', b'm', b'o', b'd', b'o', b'r', b'o', b'!'
    ];

/// BLE scan data
const SCAN_DATA: &[u8; 4] = &[0x03, 0x03, 0x09, 0x18];

/// BLE GATT server
#[nrf_softdevice::gatt_server]
pub struct Server {
    /// Configuration service
    pub config: ConfigService,
}

/// GATT server task
#[embassy_executor::task]
pub async fn ble_server_task(server: Server, sd: &'static Softdevice) {
    info!("Bluetooth ON!");

    let config = peripheral::Config::default();
    let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
        adv_data: ADV_DATA,
        scan_data: SCAN_DATA,
    };

    loop {
        match peripheral::advertise_connectable(sd, adv, &config).await {
            Ok(conn) => {
                if let Err(e) = gatt_server::run(&conn, &server, |e| match e {
                    ServerEvent::Config(ConfigServiceEvent::BytesWrite(vec)) => {
                        CONFIG_SIGNAL.signal(vec);
                    }
                })
                .await
                {
                    error!("{:?}", e);
                }
            }
            Err(e) => error!("{:?}", e),
        }
    }
}
