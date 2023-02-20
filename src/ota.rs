use embedded_svc::ota::{FirmwareInfo, FirmwareInfoLoader};
#[cfg(not(feature = "embedded"))]
use embedded_svc::{
    http::Method,
    io::{Read, ReadExactError},
};
use esp_idf_svc::errors::EspIOError;
#[cfg(not(feature = "embedded"))]
use esp_idf_svc::http::client;
use esp_idf_svc::ota::{EspFirmwareInfoLoader, EspOta};
use esp_idf_sys::EspError;
#[cfg(not(feature = "embedded"))]
use esp_idf_sys::{esp_app_desc_t, esp_image_header_t, esp_image_segment_header_t};
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to initiate firmware update: an OTA is already running: {0}")]
    SecondOta(#[source] EspError),
    #[error("Failed to initiate firmware update: {0}")]
    OtaStart(#[source] EspError),
    #[error("Failed to handle firmware info: {0}")]
    HandleFwInfo(#[source] EspIOError),
    #[error("The update file is too big: {0} > {MAX_FW_SIZE}")]
    TooBigFw(usize),
    #[error("Failed to wite FW to an ota partition: {0}")]
    OtaWrite(#[source] EspError),
    #[error("Failed to finish the update: {0}")]
    OtaComplete(#[source] EspError),
    #[error("Failed to determine boot slot: {0}")]
    BootSlot(#[source] EspError),
    #[error("Failed to determine running slot: {0}")]
    RunningSlot(#[source] EspError),
    #[error("Failed to determine update slot: {0}")]
    UpdateSlot(#[source] EspError),

    #[cfg(not(feature = "embedded"))]
    #[error("Failed to initiate OTA request: {0}")]
    OtaRequest(#[source] EspError),
    #[cfg(not(feature = "embedded"))]
    #[error("Failed to handle OTA request: {0}")]
    SubmitResp(#[source] EspError),
    #[cfg(not(feature = "embedded"))]
    #[error("Failed to abort the update: {0}")]
    OtaAbort(#[source] EspError),
    #[cfg(not(feature = "embedded"))]
    #[error("Failed to read update")]
    ReadExact(ReadExactError<EspIOError>),
    #[cfg(not(feature = "embedded"))]
    #[error("Failed to setup HTTP client: {0}")]
    HttpClientSetup(#[source] EspError),
}

/// Best that we can is to split 3MB flash by 2 for 4MB flash board.
const MAX_FW_SIZE: usize = 1_572_864;

#[cfg(not(feature = "embedded"))]
const FW_INFO_SIZE: usize = std::mem::size_of::<esp_app_desc_t>()
    + std::mem::size_of::<esp_image_header_t>()
    + std::mem::size_of::<esp_image_segment_header_t>();

#[cfg(feature = "embedded")]
pub fn perform_embedded_update() -> Result<(), Error> {
    let update = include_bytes!("../ota.bin");
    if update.len() > MAX_FW_SIZE {
        return Err(Error::TooBigFw(update.len()));
    }
    let mut ota = EspOta::new().map_err(Error::SecondOta)?;

    print_ota_info(&ota, update)?;

    let ota_update = ota.initiate_update().map_err(Error::OtaStart)?;

    ota_update.write(update).map_err(Error::OtaWrite)?;
    ota_update.complete().map_err(Error::OtaComplete)
}

#[cfg(not(feature = "embedded"))]
pub fn perform_ota_update() -> Result<(), Error> {
    let mut response = ota_request()?;

    let mut ota_bytes = vec![0; FW_INFO_SIZE];
    response
        .read_exact(&mut ota_bytes)
        .map_err(Error::ReadExact)?;

    let mut ota = EspOta::new().map_err(Error::SecondOta)?;
    print_ota_info(&ota, &ota_bytes)?;

    let ota_update = ota.initiate_update().map_err(Error::OtaStart)?;
    ota_update.write(&ota_bytes).map_err(Error::OtaWrite)?;

    // handle rest of the update faster
    let mut total_bytes_read = FW_INFO_SIZE;
    ota_bytes.resize(1024, 0);

    while let Ok(bytes_read) = response.read(&mut ota_bytes) {
        log::trace!("Read {bytes_read} bytes");

        if bytes_read == 0 {
            break;
        } else {
            total_bytes_read += bytes_read;

            if total_bytes_read > MAX_FW_SIZE {
                ota_update.abort().map_err(Error::OtaAbort)?;
                return Err(Error::TooBigFw(total_bytes_read));
            }
            ota_update.write(&ota_bytes).map_err(Error::OtaWrite)?;
        }
    }
    info!("OTA size: {total_bytes_read}");

    ota_update.complete().map_err(Error::OtaComplete)
}

#[cfg(not(feature = "embedded"))]
fn ota_request() -> Result<esp_idf_svc::http::client::EspHttpConnection, Error> {
    let mut client = client::EspHttpConnection::new(&client::Configuration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })
    .map_err(Error::HttpClientSetup)?;

    let ota_url = env!("OTA_LINK");

    info!("About to download an OTA from {ota_url}");
    client
        .initiate_request(Method::Get, ota_url, &[])
        .map_err(Error::OtaRequest)?;

    client.initiate_response().map_err(Error::SubmitResp)?;
    Ok(client)
}

fn print_ota_info(ota: &EspOta, ota_bytes: &[u8]) -> Result<(), Error> {
    let ota_info = fw_info(ota_bytes).map_err(Error::HandleFwInfo)?;
    info!("OTA info: {ota_info:?}");

    info!("initiating OTA update");

    let slot = ota.get_update_slot().map_err(Error::UpdateSlot)?;
    info!("Writing OTA to partition {slot:?}");

    let boot_slot = ota.get_boot_slot().map_err(Error::BootSlot)?;
    info!("Boot slot: {boot_slot:?}");

    let running_slot = ota.get_running_slot().map_err(Error::RunningSlot)?;
    info!("Running slot: {running_slot:?}");

    Ok(())
}

fn fw_info(update: &[u8]) -> Result<FirmwareInfo, EspIOError> {
    let mut esp_fw_loader_info = EspFirmwareInfoLoader::new();
    esp_fw_loader_info.load(update)?;
    esp_fw_loader_info.get_info()
}
