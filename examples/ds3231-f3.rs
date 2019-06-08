//! Stores the date and time on a DS3231 real-time clock (RTC).
//! Then reads the date and time repeatedly and if everything but the
//! seconds match, blinks LED 0.
//! After 1 minute it will stop blinking as the minutes will not match
//! anymore.
//!
//! This example is runs on the STM32F3 Discovery board using I2C1.
//!
//! ```
//! F3  <-> DS3231
//! GND <-> GND
//! +5V <-> +5V
//! PB7 <-> SDA
//! PB6 <-> SCL
//! ```
//!
//! Run with:
//! `cargo run --example ds3231-f3 --target thumbv7em-none-eabihf`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

// panic handler
extern crate panic_semihosting;

use cortex_m_rt::entry;
use f3::{
    hal::{delay::Delay, i2c::I2c, prelude::*, stm32f30x},
    led::Led,
};

use ds323x::{DateTime, Ds323x, Hours};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);

    let mut rtc = Ds323x::new_ds3231(i2c);
    let begin = DateTime {
        year: 2019,
        month: 1,
        day: 2,
        weekday: 3,
        hour: Hours::H24(4),
        minute: 5,
        second: 6,
    };
    rtc.set_datetime(&begin).unwrap();
    loop {
        let now = rtc.get_datetime().unwrap();
        if now.year == begin.year
            && now.month == begin.month
            && now.day == begin.day
            && now.weekday == begin.weekday
            && now.hour == begin.hour
            && now.minute == begin.minute
        {
            // as we do not compare the seconds, this will blink for one
            // minute and then stop.
            led.on();
            delay.delay_ms(500_u16);
            led.off();
            delay.delay_ms(500_u16);
        }
    }
}