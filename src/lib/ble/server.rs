//! BLE Server Config and Tasks
use super::services::*;
use crate::types::CONFIG_SIGNAL;
use defmt::*;
use nrf_softdevice::{
    ble::{gatt_server, peripheral},
    raw::{
        BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE, BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_COMPLETE,
        BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME, BLE_GAP_AD_TYPE_FLAGS,
    },
    Softdevice,
};

/// BLE advertising data
#[rustfmt::skip]
const ADV_DATA: &[u8; 14] =
    &[
        0x02, BLE_GAP_AD_TYPE_FLAGS as u8, BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x0a, BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8, b'P', b'o', b'm', b'o', b'd', b'o', b'r', b'o', b'!'
    ];

/// BLE scan response data
#[rustfmt::skip]
const SCAN_RESPONSE_DATA: &[u8; 18] = &[
    // AD length
    0x11, 
    // AD type
    BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_COMPLETE as u8,
    // AD data
    // Configuration service UUID: 11111111-1111-1111-1111-111111111122. This has to be sent in little endian order.
    0x22, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
];

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
        scan_data: SCAN_RESPONSE_DATA,
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
