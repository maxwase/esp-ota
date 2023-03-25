#[cfg(not(feature = "embedded"))]
use esp_idf_sys::EspError;
use log::error;
use thiserror::Error;

mod ota;
#[cfg(not(feature = "embedded"))]
mod wifi;

#[derive(Error, Debug)]
enum Error {
    #[error("Update error: {0}")]
    Ota(#[from] crate::ota::Error),

    #[cfg(not(feature = "embedded"))]
    #[error("Wi-Fi setup error: {0}")]
    WiFi(#[from] wifi::Error),

    #[cfg(not(feature = "embedded"))]
    #[error("Failed to get `EspSysLoopStack`: {0}")]
    EspSysLoopStack(#[source] EspError),

    #[cfg(not(feature = "embedded"))]
    #[error("Failed to get `EspDefaultNvs`: {0}")]
    EspDefaultNvs(#[source] EspError),
}

fn main() -> Result<(), Error> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    #[cfg(not(feature = "embedded"))]
    let update_result = setup_wifi();
    #[cfg(not(feature = "embedded"))]
    let update_result = ota::perform_ota_update();

    #[cfg(feature = "embedded")]
    let update_result = ota::perform_embedded_update();

    if let Err(e) = update_result {
        error!("Error: {e}");
    }

    // OTA has finished successfully, reboot
    unsafe { esp_idf_sys::esp_restart() }
}

#[cfg(not(feature = "embedded"))]
fn setup_wifi() -> Result<(), Error> {
    use embedded_svc::wifi::{ClientConfiguration, Configuration};
    use esp_idf_hal::modem::WifiModem;
    use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

    use wifi::set_wifi_configuration;

    let wifi_modem = unsafe { WifiModem::new() };
    let sys_loop = EspSystemEventLoop::take().map_err(Error::EspSysLoopStack)?;
    let nvs = EspDefaultNvsPartition::take().map_err(Error::EspDefaultNvs)?;
    let mut wifi =
        EspWifi::new(wifi_modem, sys_loop.clone(), Some(nvs)).map_err(wifi::Error::Setup)?;

    set_wifi_configuration(
        &mut wifi,
        &sys_loop,
        Configuration::Client(ClientConfiguration {
            ssid: env!("ESP_SSID").into(),
            password: env!("ESP_PASSWD").into(),
            ..Default::default()
        }),
    )
    .map_err(Into::into)
}
