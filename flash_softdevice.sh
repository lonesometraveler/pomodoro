#!/bin/bash
probe-rs-cli erase --chip nRF52833_xxAA && \
probe-rs-cli download --chip nRF52833_xxAA --format hex s140_nrf52_7.3.0/s140_nrf52_7.3.0_softdevice.hex