use embedded_storage_async::nor_flash::*;
use nrf_softdevice::{Flash, FlashError};

/// Flash Storage
pub struct FlashStorage {
    start_addr: u32,
    end_addr: u32,
}

impl FlashStorage {
    /// Creates Storage by specifying start and end addresses
    pub fn new(start_addr: u32, end_addr: u32) -> Self {
        FlashStorage {
            start_addr,
            end_addr,
        }
    }

    /// Reads from flash
    pub async fn read(&self, f: &mut Flash, buf: &mut [u8]) -> Result<(), FlashError> {
        f.read(self.start_addr, buf).await
    }

    /// Writes to flash
    pub async fn write(&self, f: &mut Flash, bytes: &mut [u8]) -> Result<(), FlashError> {
        f.erase(self.start_addr, self.end_addr).await?;
        f.write(self.start_addr, bytes).await
    }
}
