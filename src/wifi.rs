use std::time::Duration;

use embedded_svc::wifi::{Configuration, Wifi};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use esp_idf_sys::EspError;
use log::{error, trace};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to set up ESP WiFi {0}")]
    Setup(#[source] EspError),
    #[error("Failed to start WiFi {0}")]
    Start(#[source] EspError),
    #[error("Failed to connect to WiFi {0}")]
    Connect(#[source] EspError),
    #[error("Failed to wait WiFi set up {0}")]
    Wait(#[source] EspError),
    #[error("WiFi did not start")]
    WaitStart,
    #[error("WiFi did not connect")]
    WaitConnect,
    #[error("Failed to configure ESP WiFI {0}")]
    Configuration(#[source] EspError),
}

/// Setup wifi configuration.
pub fn set_wifi_configuration(
    wifi: &mut EspWifi,
    sys_loop: &EspSystemEventLoop,
    configuration: Configuration,
) -> Result<(), Error> {
    wifi.set_configuration(&configuration)
        .map_err(Error::Configuration)?;
    wifi.start().map_err(Error::Start)?;

    if !WifiWait::new(sys_loop)
        .map_err(Error::Wait)?
        .wait_with_timeout(Duration::from_secs(20), || {
            wifi.is_started().unwrap_or_default()
        })
    {
        return Err(Error::WaitStart);
    }

    trace!("Wifi started");

    wifi.connect().map_err(Error::Connect)?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), sys_loop)
        .map_err(Error::Wait)?
        .wait_with_timeout(Duration::from_secs(20), || {
            wifi.is_connected().unwrap_or_default()
                && wifi
                    .sta_netif()
                    .get_ip_info()
                    .map(|info| !info.ip.is_unspecified())
                    .unwrap_or_default()
        })
    {
        return Err(Error::WaitConnect);
    }

    trace!("Wifi connected");
    Ok(())
}
