//! BLE GATT services

/// Configuration service
#[nrf_softdevice::gatt_service(uuid = "11111111-1111-1111-1111-111111111111")]
pub struct ConfigService {
    #[characteristic(uuid = "11111111-1111-1111-1111-111111111111", read, write)]
    pub bytes: heapless::Vec<u8, 32>,
}
