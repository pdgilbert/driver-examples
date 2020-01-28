//! Measure the voltages with an ADS1015 analog/digital
//! converter and print them to an SSD1306 OLED display.
//!
//! You can see further explanations about this device and how this example
//! works here:
//!
//! https://blog.eldruin.com/ads1x1x-analog-to-digital-converter-driver-in-rust/
//!
//! This example is runs on the STM32F1 "BluePill" board using I2C1.
//!
//! ```
//! BP  <-> ADS1015 <-> Display
//! GND <-> GND     <-> GND
//! +5V <-> +5V     <-> +5V
//! PB9 <-> SDA     <-> SDA
//! PB8 <-> SCL     <-> SCL
//! ```
//!
//! For example you can create a simple voltage divider with resistors.
//! The values do not matter much but it is nicer to understand if they are
//! all the same as the voltage will be divided equally.
//! I used 3 resistors of 20KOhm and the inputs of the ADC were connected
//! as follows:
//!
//! ```
//!       ADS1015
//! +5V <-> A0
//!  |
//!  R3
//!  |  <-> A1
//!  R2
//!  |  <-> A2
//!  R1
//!  |
//! GND <-> A3
//! ```
//!
//! You can see an image of this [here](https://github.com/eldruin/driver-examples/blob/master/media/ads1015-voltage-divider.jpg).
//!
//! With this setup we should get the reading for +5V on channel A0,
//! the reading for GND on channel A3 and A1 and A2 equally spaced in between
//! (within resistence tolerances).
//!
//! I get these values:
//! Channel 0: 1575
//! Channel 1: 1051
//! Channel 2: 524
//! Channel 3: 0
//!
//! We can calculate the relations and voltage that correspond to each channel if we
//! assume that 1575 corresponds to 5V.
//!                         Factor        Voltage
//! Channel 0: 1575 / 1575 = 1     * 5V =   5V
//! Channel 1: 1051 / 1575 = 0.667 * 5V =  3.34V
//! Channel 2: 524  / 1575 = 0.333 * 5V =  1.66V
//! Channel 3: 0    / 1575 = 0     * 5V =   0V
//!
//! As you can see, the voltage was divided equally by all resistors.
//!
//! Run with:
//! `cargo run --example ads1015-adc-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{fonts::Font6x8, prelude::*};
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use panic_semihosting as _;
use ssd1306::{prelude::*, Builder};
use stm32f1xx_hal::{
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 100_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut delay = Delay::new(cp.SYST, clocks);

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let mut adc = Ads1x1x::new_ads1015(manager.acquire(), SlaveAddr::default());
    // need to be able to measure [0-5V]
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.set_high().unwrap();
        delay.delay_ms(50_u16);
        led.set_low().unwrap();
        delay.delay_ms(50_u16);

        // Read voltage in all channels
        let values = [
            block!(adc.read(&mut AdcChannel::SingleA0)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA1)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA2)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA3)).unwrap_or(8091),
        ];

        let mut lines: [heapless::String<heapless::consts::U32>; 4] = [
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
        ];

        // write some extra spaces after the number to clear up when the numbers get smaller
        for i in 0..values.len() {
            write!(lines[i], "Channel {}: {}    ", i, values[i]).unwrap();
            disp.draw(
                Font6x8::render_str(&lines[i])
                    .with_stroke(Some(1u8.into()))
                    .translate(Coord::new(0, i as i32 * 16))
                    .into_iter(),
            );
        }
        disp.flush().unwrap();
    }
}