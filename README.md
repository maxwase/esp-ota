# ESP32 OTA update

This repo is an example of how to configure an OTA update for ESP32 using the `esp-idf-svc` crate.

The default scenario is to connect to a Wi-Fi (`ESP_SSID` & `ESP_PASSWD` envs) and download an OTA image from `OTA_LINK` env. 
`embedded` feature overrides OTA image downloading with an OTA image embedded in the firmware (🪆 Matryoshka scenario)

Before compiling the project with the `embedded` feature, make sure you have `ota.bin` in the root of the repo, you can compile it from the blinky example using `build_ota.sh`.
This way you can achieve double OTA testing: flash the firmware -> that downloads and flashes an OTA with an embedded OTA -> that will flash an embedded blinky example!

## Build & Flash
```sh
export ESP_SSID="NAME"
export ESP_PASSWD="PASSWD"
export OTA_LINK="LINK"
cargo espflash flash --release --monitor
```
## Testing

This script will execute the following tasks:
1. Build the example blinky as OTA
2. Build the embedded updater with blinky as OTA included
2. Build and Flash the embedded updater with an embedded updater as OTA, which has blinky as OTA
4. After two successful embedded OTA updates have been completed, the LED should blink
   
```sh
./build_ota test
cargo espflash flash --release --monitor --features=embedded
```

## Troubleshooting

* If you see something like `thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Utf8Error { valid_up_to: 277, error_len: Some(1) }'`, then check that your ENVs does not contain escape-characters
* If you see something like `Error while running FlashDeflateBegin command`, then it might be caused by too large `ota.bin`
* If OTA is failing, try erasing `otadata`, `ota_0`, and `ota_1` partitions. That can be done by `otatool.py erase_otadata`  and `otatool.py erase_ota_partition --name ota_X` from [esp-idf repo](https://github.com/espressif/esp-idf)
* If OTA is failing, try `esptool.py --chip esp32c3 image_info ota.bin`, it might give you some errors
* If you see some error that says that firmware checksum is incorrect or the size is wrong, check that LTO is disabled ;)