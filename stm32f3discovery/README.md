# Additional example programs for several rust drivers

[![Build Status](https://travis-ci.org/eldruin/driver-examples.svg?branch=master)](https://travis-ci.org/eldruin/driver-examples)

This repository includes examples of using devices through these drivers:

| Device driver | Description                                               | Interface | Introductory blog post            |
|---------------|-----------------------------------------------------------|-----------|-----------------------------------|
|[Ad983x]       | Waveform generator / direct digital synthesizer (DDS).    | SPI       | [Intro blog post][blog-ad983x]    |
|[Ads1x1x]      | 12/16-bit Analog-to-digital (ADC) converters.             | I2C       | [Intro blog post][blog-ads1x1x]   |
|[Apds9960]     | Digital proximity, ambient light, RGB and gesture sensor. | I2C       |                                   |
|[Ds1307]       | Real-time clock (RTC).                                    | I2C       | [Intro blog post][blog-ds1307]    |
|[Ds323x]       | Extremely accurate real-time clock (RTC).                 | I2C / SPI |                                   |
|[Kxcj9]        | Tri-axis accelerometer                                    | I2C       |                                   |
|[Eeprom24x]    | 24x series serial EEPROM devices.                         | I2C       | [Intro blog post][blog-eeprom24x] |
|[Lm75]         | Temperature sensor and thermal watchdog.                  | I2C       |                                   |
|[Max3010x]     | Pulse oximeter and heart-rate sensor.                     | I2C       |                                   |
|[Mcp4x]        | Digital potentiometers.                                   | SPI       |                                   |
|[Mcp49x]       | 8/10/12-bit Digital-to-analog (DAC) converters.           | SPI       |                                   |
|[Pcf857x]      | 8/16-pin I/O port expanders.                              | I2C       |                                   |
|[Pwm-pca9685]  | 16-pin PWM port expander / LED driver.                    | I2C       |                                   |
|[Tcs3472]      | RGBW light color sensor with IR filter.                   | I2C       |                                   |
|[Tmp006]       | Non-contact infrared (IR) thermopile temperature sensor.  | I2C       | [Intro blog post][blog-tmp006]    |
|[Tmp1x2]       | Temperature sensors.                                      | I2C       | [Intro blog post][blog-tmp1x2]    |
|[Veml6040]     | RGBW light color sensor.                                  | I2C       |                                   |
|[Veml6075]     | Ultraviolet A (UVA) and B (UVB) light sensor.             | I2C       |                                   |
|[W25]          | Winbond's W25 serial flash memory devices.                | SPI       |                                   |

These examples use the STM32F3Discovery board. At the beginning of each example the setup
and behavior is described. Many of them also use an SSD1306 OLED display.
You can get the modules used here on [AliExpress] generally for a very small price.

For example, to run the f3-mcp41x example:
First, connect your discovery board per USB, then connect OpenOCD in a terminal with:
```
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
```

Then on another terminal run:
```
git clone https://github.com/eldruin/driver-examples
cd driver-examples
cargo run --example f3-mcp41x
```

## License

Licensed under either of

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
[Lm75]: https://crates.io/crates/lm75
[Max3010x]: https://crates.io/crates/max3010x
[Mcp4x]: https://crates.io/crates/mcp4x
[Mcp49x]: https://github.com/eldruin/mcp49x-rs
[Pcf857x]: https://crates.io/crates/pcf857x
[Pwm-pca9685]: https://crates.io/crates/pwm-pca9685
[Tcs3472]: https://crates.io/crates/tcs3472
[Tmp006]: https://crates.io/crates/tmp006
[Tmp1x2]: https://crates.io/crates/tmp1x2
[Veml6040]: https://crates.io/crates/veml6040
[Veml6075]: https://crates.io/crates/veml6075
[W25]: https://github.com/eldruin/w25-rs

[blog-ad983x]: https://blog.eldruin.com/ad983x-waveform-generator-dds-driver-in-rust/
[blog-ads1x1x]: https://blog.eldruin.com/ads1x1x-analog-to-digital-converter-driver-in-rust/
[blog-ds1307]: https://blog.eldruin.com/ds1307-real-time-clock-rtc-driver-in-rust/
[blog-eeprom24x]: https://blog.eldruin.com/24x-serial-eeprom-driver-in-rust/
[blog-tmp006]: https://blog.eldruin.com/tmp006-contact-less-infrared-ir-thermopile-driver-in-rust/
[blog-tmp1x2]: https://blog.eldruin.com/tmp1x2-temperature-sensor-driver-in-rust/

[AliExpress]: https://www.aliexpress.com