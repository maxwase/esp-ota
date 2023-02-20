#!/bin/sh

ProgName=$(basename $0)
  
help(){
    echo "Usage: $ProgName <subcommand> [options]\n"
    echo "Subcommands:"
    echo "    example   bild example blinky as OTA"
    echo "    self   bild self as OTA"
    echo "    test   build matryoshka OTA"
    echo ""
}

case $1 in
    self)
        cargo espflash save-image --release ESP32-C3 ota.bin $2
    ;;
    example)
        cargo espflash save-image --example blinky --release ESP32-C3 ota.bin
    ;;
    test)
        # Compile blinky ota.bin 
        ./$ProgName example
        # Compile updater ota.bin with blinky inside
       ./$ProgName self "--features embedded"
    ;;

    "" | "-h" | "--help" | *)
        help
    ;;
esac