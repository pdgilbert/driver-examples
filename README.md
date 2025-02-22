# Additional example programs for several rust drivers

[![Build Status](https://github.com/eldruin/driver-examples/workflows/Build/badge.svg)](https://github.com/eldruin/driver-examples/actions?query=workflow%3ABuild)

This repository includes examples of using devices through these drivers:

| Device driver    | Description                                               | Interface | Introductory blog post            |
|------------------|-----------------------------------------------------------|-----------|-----------------------------------|
|[Ad983x]          | Waveform generator / direct digital synthesizer (DDS).    | SPI       | [Intro blog post][blog-ad983x]    |
|[Ads1x1x]         | 12/16-bit Analog-to-digital (ADC) converters.             | I2C       | [Intro blog post][blog-ads1x1x]   |
|[Apds9960]        | Digital proximity, ambient light, RGB and gesture sensor. | I2C       |                                   |
|[Ds1307]          | Real-time clock (RTC) / calendar.                         | I2C       | [Intro blog post][blog-ds1307]    |
|[Ds323x]          | Extremely accurate real-time clock (RTC) / calendar.      | I2C / SPI |                                   |
|[Eeprom24x]       | 24x series serial EEPROM devices.                         | I2C       | [Intro blog post][blog-eeprom24x] |
|[Embedded-Ccs811] | Digital gas sensor for monitoring indoor air quality.     | I2C       | [Intro blog post][blog-ccs811]    |
|[Hdc20xx]         | Temperature and humidity sensors.                         | I2C       |                                   |
|[iAQ-Core]        | Indoor air quality sensor.                                | I2C       |                                   |
|[Isl29125]        | RGB color light sensor with IR blocking filter.           | I2C       |                                   |
|[Kxcj9]           | Tri-axis MEMS accelerometer.                              | I2C       | [Intro blog post][blog-kxcj9]     |
|[Lm75]            | Temperature sensor and thermal watchdog.                  | I2C       |                                   |
|[Lsm303agr]       | Tri-axis accelerometer and tri-axis magnetometer.         | I2C / SPI |                                   |
|[Max170xx]        | Fuel-gauge for lithium-ion (Li+) batteries.               | I2C       |                                   |
|[Max3010x]        | Pulse oximeter and heart-rate sensor.                     | I2C       |                                   |
|[Max44009]        | Ambient light sensor.                                     | I2C       |                                   |
|[Mcp4x]           | Digital potentiometers.                                   | SPI       |                                   |
|[Mcp49xx]         | 8/10/12-bit Digital-to-analog (DAC) converters.           | SPI       |                                   |
|[Mcp794xx]        | Real-time clock (RTC) / calendar.                         | I2C       | [Intro blog post][blog-mcp794xx]  |
|[Mlx9061x]        | Non-contact infrared (IR) thermometer.                    | I2C       |                                   |
|[Mma8x5x]         | Tri-axis MEMS accelerometers.                             | I2C       |                                   |
|[Opt300x]         | Ambient light sensor.                                     | I2C       | [Intro blog post][blog-opt300x]   |
|[Pcf857x]         | 8/16-pin I/O port expanders.                              | I2C       |                                   |
|[Pwm-Pca9685]     | 16-pin PWM port expander / LED driver.                    | I2C       | [Intro blog post][blog-pca9685]   |
|[Si4703]          | FM radio turners (receivers).                             | I2C       | [Intro blog post][blog-si4703]    |
|[Tcs3472]         | RGBW light color sensor with IR filter.                   | I2C       |                                   |
|[Tmp006]          | Non-contact infrared (IR) thermopile temperature sensor.  | I2C       | [Intro blog post][blog-tmp006]    |
|[Tmp1x2]          | Temperature sensors.                                      | I2C       | [Intro blog post][blog-tmp1x2]    |
|[Veml6030]        | Ambient light sensor.                                     | I2C       | [Intro blog post][blog-veml6030]  |
|[Veml6040]        | RGBW light color sensor.                                  | I2C       |                                   |
|[Veml6070]        | Ultraviolet A (UVA) light sensor.                         | I2C       |                                   |
|[Veml6075]        | Ultraviolet A (UVA) and B (UVB) light sensor.             | I2C       | [Intro blog post][blog-veml6075]  |
|[W25]             | Winbond's W25 serial flash memory devices.                | SPI       |                                   |
|[Xca9548a]        | TCA9548A/PCA9548A I2C switches/multiplexers.              | I2C       |                                   |

These examples use several boards: STM32F3-Discovery, STM32F103 "Blue pill", Raspberry Pi
and Micro:bit V2. These are classified in different folders.

At the beginning of each example the setup and behavior is described.
Many of them also use an SSD1306 OLED display.
You can get most of the modules used here on [AliExpress] generally for a very small price.

These examples are guaranteed to build with the latest Rust stable release.
If you get a build error, try updating your Rust installation.

To run the examples, clone this repository, go to the appropriate folder and run
either `cargo embed` or `cargo run`. Look in the README of each folder for instructions.
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples/stm32f1-discovery
# ...
```

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[Ad983x]: https://crates.io/crates/ad983x
[Ads1x1x]: https://crates.io/crates/ads1x1x
[Apds9960]: https://crates.io/crates/apds9960
[Ds1307]: https://crates.io/crates/ds1307
[Ds323x]: https://crates.io/crates/ds323x
[Kxcj9]: https://crates.io/crates/kxcj9
[Eeprom24x]: https://crates.io/crates/eeprom24x
[Embedded-Ccs811]: https://crates.io/crates/embedded-ccs811
[Hdc20xx]: https://crates.io/crates/hdc20xx
[Isl29125]: https://crates.io/crates/isl29125
[iAQ-Core]: https://crates.io/crates/iaq-core
[Lm75]: https://crates.io/crates/lm75
[Lsm303agr]: https://crates.io/crates/lsm303agr
[Max170xx]: https://crates.io/crates/max170xx
[Max3010x]: https://crates.io/crates/max3010x
[Max44009]: https://crates.io/crates/max44009
[Mcp4x]: https://crates.io/crates/mcp4x
[Mcp49xx]: https://crates.io/crates/mcp49xx
[Mcp794xx]: https://crates.io/crates/mcp794xx
[Mlx9061x]: https://crates.io/crates/mlx9061x
[Mma8x5x]: https://crates.io/crates/mma8x5x
[Opt300x]: https://crates.io/crates/Opt300x
[Pcf857x]: https://crates.io/crates/pcf857x
[Pwm-Pca9685]: https://crates.io/crates/pwm-pca9685
[Si4703]: https://crates.io/crates/si4703
[Tcs3472]: https://crates.io/crates/tcs3472
[Tmp006]: https://crates.io/crates/tmp006
[Tmp1x2]: https://crates.io/crates/tmp1x2
[Veml6030]: https://crates.io/crates/veml6030
[Veml6040]: https://crates.io/crates/veml6040
[Veml6070]: https://crates.io/crates/veml6070
[Veml6075]: https://crates.io/crates/veml6075
[W25]: https://github.com/eldruin/w25-rs
[Xca9548a]: https://crates.io/crates/xca9548a

[blog-ad983x]: https://blog.eldruin.com/ad983x-waveform-generator-dds-driver-in-rust/
[blog-ads1x1x]: https://blog.eldruin.com/ads1x1x-analog-to-digital-converter-driver-in-rust/
[blog-ccs811]: https://blog.eldruin.com/ccs811-indoor-air-quality-sensor-driver-in-rust/
[blog-ds1307]: https://blog.eldruin.com/ds1307-real-time-clock-rtc-driver-in-rust/
[blog-eeprom24x]: https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/
[blog-kxcj9]: https://blog.eldruin.com/kxcj9-kxcjb-tri-axis-mems-accelerator-driver-in-rust/
[blog-mcp794xx]: https://blog.eldruin.com/mcp794xx-real-time-clock-rtc-driver-in-rust
[blog-opt300x]: https://blog.eldruin.com/opt300x-ambient-light-sensor-driver-in-rust/
[blog-pca9685]: https://blog.eldruin.com/pca9685-pwm-led-servo-controller-driver-in-rust/
[blog-si4703]: https://blog.eldruin.com/si4703-fm-radio-receiver-driver-in-rust/
[blog-tmp006]: https://blog.eldruin.com/tmp006-contact-less-infrared-ir-thermopile-driver-in-rust/
[blog-tmp1x2]: https://blog.eldruin.com/tmp1x2-temperature-sensor-driver-in-rust/
[blog-veml6030]: https://blog.eldruin.com/veml6030-ambient-light-sensor-driver-in-rust/
[blog-veml6075]: https://blog.eldruin.com/veml6075-uva-uvb-uv-index-light-sensor-driver-in-rust/

[AliExpress]: https://www.aliexpress.com