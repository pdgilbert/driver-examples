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
//! `cargo embed --example ads1015-adc-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use ssd1306::{prelude::*, Builder, I2CDIBuilder};

pub trait LED {
    // depending on board wiring, on may be set_high or set_low, with off also reversed
    // implementation should deal with this difference
    fn on(&mut self) -> ();
    fn off(&mut self) -> ();

    // default methods
    fn blink(&mut self, time: u16, delay: &mut Delay) -> () {
        self.on();
        delay.delay_ms(time);
        self.off()
    }
}


#[cfg(feature = "stm32f1xx")]
use stm32f1xx_hal::{
    delay::Delay,
    device::I2C1,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{BlockingI2c, DutyCycle, Mode, Pins},
    pac,
    prelude::*,
};

#[cfg(feature = "stm32f1xx")]
fn setup() -> (BlockingI2c<I2C1, impl Pins<I2C1>>, impl LED, Delay) {
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
            frequency: 100_000.hz(),
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
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let delay = Delay::new(cp.SYST, clocks);

    impl LED for PC13<Output<PushPull>> {
        fn on(&mut self) -> () {
            self.set_low().unwrap()
        }
        fn off(&mut self) -> () {
            self.set_high().unwrap()
        }
    }

    (i2c, led, delay)
}

#[cfg(feature = "stm32f3xx")] //  eg Discovery-stm32f303
use stm32f3xx_hal::{
    delay::Delay,
    gpio::{gpioe::PE9, Output, PushPull},
    i2c::{I2c, SclPin, SdaPin},
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
};

#[cfg(feature = "stm32f3xx")]
fn setup() -> (
    I2c<I2C1, (impl SclPin<I2C1>, impl SdaPin<I2C1>)>,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let delay = Delay::new(cp.SYST, clocks);

    impl LED for PE9<Output<PushPull>> {
        fn on(&mut self) -> () {
            self.set_high().unwrap()
        }
        fn off(&mut self) -> () {
            self.set_low().unwrap()
        }
    }

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    let scl = gpiob.pb6
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob.pb7
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    //    // not sure if pull up is needed
    //    scl.internal_pull_up(&mut gpiob.pupdr, true);
    //    sda.internal_pull_up(&mut gpiob.pupdr, true);

    let i2c = I2c::new(dp.I2C1, (scl, sda), 100_000.Hz(), clocks, &mut rcc.apb1);

    (i2c, led, delay) // return tuple (i2c, led, delay)
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("ADS1015 example");

    let (i2c, mut led, mut delay) = setup();

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let interface = I2CDIBuilder::new().init(manager.acquire());
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut adc = Ads1x1x::new_ads1015(manager.acquire(), SlaveAddr::default());
    // need to be able to measure [0-5V]
    adc.set_full_scale_range(FullScaleRange::Within6_144V)
        .unwrap();

    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.blink(50_u16, &mut delay);

        // Read voltage in all channels
        let values = [
            block!(adc.read(&mut AdcChannel::SingleA0)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA1)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA2)).unwrap_or(8091),
            block!(adc.read(&mut AdcChannel::SingleA3)).unwrap_or(8091),
        ];

        let mut lines: [heapless::String<32>; 4] = [
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
            heapless::String::new(),
        ];

        disp.clear();
        for i in 0..values.len() {
            write!(lines[i], "Channel {}: {}", i, values[i]).unwrap();
            Text::new(&lines[i], Point::new(0, i as i32 * 16))
                .into_styled(text_style)
                .draw(&mut disp)
                .unwrap();
        }
        disp.flush().unwrap();
    }
}
