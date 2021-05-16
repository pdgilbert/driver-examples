//! Continuously measure the eCO2 and eTVOC in the air and print it to an
//! SSD1306 OLED display.
//! In order to compensate for the ambient temperature and humidity, an HDC2080
//! sensor is used.
//!
//! Introductory blog post with some pictures here:
//! https://blog.eldruin.com/ccs811-indoor-air-quality-sensor-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using I2C1.
//!
//! ```
//! BP   <-> CCS811 <-> HDC2080 <-> Display
//! GND  <-> GND    <-> GND     <-> GND
//! 3.3V <-> VCC    <-> VCC     <-> VDD
//! PB8  <-> SCL    <-> SCL     <-> SCL
//! PB9  <-> SDA    <-> SDA     <-> SDA
//! GND  <-> nWAKE
//! 3.3V <-> RST
//! ```
//!
//! Run with:
//! `cargo embed --example ccs811-gas-voc-display-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_ccs811::{
    prelude::*, AlgorithmResult, Ccs811Awake, MeasurementMode, SlaveAddr as Ccs811SlaveAddr,
};
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use embedded_hal::digital::v2::OutputPin;
use hdc20xx::{Hdc20xx, SlaveAddr as Hdc20xxSlaveAddr};
use heapless::String;
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

// setup() does all  hal/MCU specific setup and returns generic hal device for use in main code.

#[cfg(feature = "stm32f0xx")] //  eg stm32f030xc
use stm32f0xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{I2c, SclPin, SdaPin},
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
};

#[cfg(feature = "stm32f0xx")]
fn setup() -> (
    I2c<I2C1, impl SclPin<I2C1>, impl SdaPin<I2C1>>,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let mut p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);

    let gpiob = p.GPIOB.split(&mut rcc); // for i2c scl and sda

    let (scl, sda) = cortex_m::interrupt::free(move |cs| {
        (
            gpiob.pb8.into_alternate_af1(cs), // scl on PB8
            gpiob.pb7.into_alternate_af1(cs), // sda on PB7
        )
    });

    let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz(), &mut rcc);

    let delay = Delay::new(cp.SYST, &rcc);

    // led
    let gpioc = p.GPIOC.split(&mut rcc);
    let led = cortex_m::interrupt::free(move |cs| gpioc.pc13.into_push_pull_output(cs));

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

#[cfg(feature = "stm32f1xx")]
use stm32f1xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{BlockingI2c, DutyCycle, Mode, Pins},
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
};

#[cfg(feature = "stm32f1xx")]
fn setup() -> (BlockingI2c<I2C1, impl Pins<I2C1>>, impl LED, Delay) {
    let cp = CorePeripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

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

    let scl = gpiob
        .pb6
        .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob
        .pb7
        .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    //    // not sure if pull up is needed
    //    scl.internal_pull_up(&mut gpiob.pupdr, true);
    //    sda.internal_pull_up(&mut gpiob.pupdr, true);

    let i2c = I2c::new(dp.I2C1, (scl, sda), 100_000.Hz(), clocks, &mut rcc.apb1);

    (i2c, led, delay)
}

#[cfg(feature = "stm32f4xx")] // eg Nucleo-64  stm32f411
use stm32f4xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{I2c, Pins},
    pac::{CorePeripherals, Peripherals, I2C2},
    prelude::*,
};

#[cfg(feature = "stm32f4xx")]
fn setup() -> (I2c<I2C2, impl Pins<I2C2>>, impl LED, Delay) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpiob = p.GPIOB.split(); // for i2c

    // can have (scl, sda) using I2C1  on (PB8  _af4, PB9 _af4) or on  (PB6 _af4, PB7 _af4)
    //     or   (scl, sda) using I2C2  on (PB10 _af4, PB3 _af9)

    let scl = gpiob.pb10.into_alternate_af4().set_open_drain(); // scl on PB10
    let sda = gpiob.pb3.into_alternate_af9().set_open_drain(); // sda on PB3

    let i2c = I2c::new(p.I2C2, (scl, sda), 400.khz(), clocks);

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let gpioc = p.GPIOC.split();
    let led = gpioc.pc13.into_push_pull_output();

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

#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{BlockingI2c, Mode, PinScl, PinSda},
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
};

#[cfg(feature = "stm32f7xx")]
fn setup() -> (
    BlockingI2c<I2C1, impl PinScl<I2C1>, impl PinSda<I2C1>>,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let scl = gpiob.pb8.into_alternate_af4().set_open_drain(); // scl on PB8
    let sda = gpiob.pb9.into_alternate_af4().set_open_drain(); // sda on PB9

    let i2c = BlockingI2c::i2c1(
        p.I2C1,
        (scl, sda),
        //400.khz(),
        Mode::Fast {
            frequency: 400_000.hz(),
        },
        clocks,
        &mut rcc.apb1,
        1000,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let led = gpioc.pc13.into_push_pull_output(); // led on pc13

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

#[cfg(feature = "stm32h7xx")]
use stm32h7xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::I2c,
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
};

#[cfg(feature = "stm32h7xx")]
fn setup() -> (I2c<I2C1>, impl LED, Delay) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let pwr = p.PWR.constrain();
    let vos = pwr.freeze();
    let rcc = p.RCC.constrain();
    let ccdr = rcc.sys_ck(160.mhz()).freeze(vos, &p.SYSCFG);
    let clocks = ccdr.clocks;

    let gpiob = p.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioc = p.GPIOC.split(ccdr.peripheral.GPIOC);

    let scl = gpiob.pb8.into_alternate_af4().set_open_drain(); // scl on PB8
    let sda = gpiob.pb9.into_alternate_af4().set_open_drain(); // sda on PB9

    let i2c = p
        .I2C1
        .i2c((scl, sda), 400.khz(), ccdr.peripheral.I2C1, &clocks);

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let led = gpioc.pc13.into_push_pull_output(); // led on pc13

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

#[cfg(feature = "stm32l0xx")]
use stm32l0xx_hal::{
    delay::Delay,
    gpio::{
        gpiob::{PB8, PB9},
        gpioc::PC13,
        OpenDrain, Output, PushPull,
    },
    i2c::I2c,
    pac::{CorePeripherals, Peripherals, I2C1},
    prelude::*,
    rcc, // for ::Config but note name conflict with serial
};

#[cfg(feature = "stm32l0xx")]
fn setup() -> (
    I2c<I2C1, PB9<Output<OpenDrain>>, PB8<Output<OpenDrain>>>,
    //I2c<I2C1, impl Pins<I2C1>>,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi16());
    let clocks = rcc.clocks;

    let gpiob = p.GPIOB.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);

    // could also have scl on PB6, sda on PB7
    //BlockingI2c::i2c1(
    let scl = gpiob.pb8.into_open_drain_output(); // scl on PB8
    let sda = gpiob.pb9.into_open_drain_output(); // sda on PB9

    let i2c = p.I2C1.i2c(sda, scl, 400.khz(), &mut rcc);

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let led = gpioc.pc13.into_push_pull_output(); // led on pc13 with on/off

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

#[cfg(feature = "stm32l1xx")] // eg  Discovery STM32L100 and Heltec lora_node STM32L151CCU6
use stm32l1xx_hal::{
    delay::Delay,
    gpio::{gpiob::PB6, Output, PushPull},
    i2c::{I2c, Pins},
    prelude::*,
    rcc, // for ::Config but avoid name conflict with serial
    stm32::{CorePeripherals, Peripherals, I2C1},
    //gpio::{gpiob::{PB8, PB9}, Output, OpenDrain, },
};

#[cfg(feature = "stm32l1xx")]
fn setup() -> (I2c<I2C1, impl Pins<I2C1>>, impl LED, Delay) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi());
    let clocks = rcc.clocks;

    let gpiob = p.GPIOB.split();

    // could also have scl,sda  on PB6,PB7 or on PB10,PB11
    let scl = gpiob.pb8.into_open_drain_output(); // scl on PB8
    let sda = gpiob.pb9.into_open_drain_output(); // sda on PB9

    let i2c = p.I2C1.i2c((scl, sda), 400.khz(), &mut rcc);

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let led = gpiob.pb6.into_push_pull_output(); // led on pb6

    impl LED for PB6<Output<PushPull>> {
        fn on(&mut self) -> () {
            self.set_high().unwrap()
        }
        fn off(&mut self) -> () {
            self.set_low().unwrap()
        }
    }

    (i2c, led, delay)
}

#[cfg(feature = "stm32l4xx")]
use stm32l4xx_hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Output, PushPull},
    i2c::{I2c, SclPin, SdaPin},
    pac::{CorePeripherals, Peripherals, I2C2},
    prelude::*,
};

#[cfg(feature = "stm32l4xx")]
fn setup() -> (
    I2c<I2C2, (impl SclPin<I2C2>, impl SdaPin<I2C2>)>,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    // following ttps://github.com/stm32-rs/stm32l4xx-hal/blob/master/examples/i2c_write.rs
    let mut scl = gpiob
        .pb10
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper); // scl on PB10
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    let mut sda = gpiob
        .pb11
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper); // sda on PB11
    sda.internal_pull_up(&mut gpiob.pupdr, true);
    let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    let i2c = I2c::i2c2(p.I2C2, (scl, sda), 400.khz(), clocks, &mut rcc.apb1r1);

    let mut gpioc = p.GPIOC.split(&mut rcc.ahb2);

    let delay = Delay::new(cp.SYST, clocks);

    // led
    let led = gpioc
        .pc13
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper); // led on pc13

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

// End of hal/MCU specific setup. Following should be generic code.

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("CCS811/HDC2080 example");

    let (i2c, mut led, mut delay) = setup();

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let interface = I2CDIBuilder::new().init(manager.acquire());
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let text_style = TextStyleBuilder::new(Font6x8)
        .text_color(BinaryColor::On)
        .build();

    let mut hdc2080 = Hdc20xx::new(manager.acquire(), Hdc20xxSlaveAddr::default());
    let mut ccs811 = Ccs811Awake::new(manager.acquire(), Ccs811SlaveAddr::default());
    ccs811.software_reset().unwrap();
    delay.delay_ms(10_u16);
    let mut lines: [String<32>; 4] = [String::new(), String::new(), String::new(), String::new()];

    let mut ccs811 = ccs811.start_application().ok().unwrap();
    let mut env = block!(hdc2080.read()).unwrap();
    ccs811
        .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
        .unwrap();
    ccs811.set_mode(MeasurementMode::ConstantPower1s).unwrap();

    let default = AlgorithmResult {
        eco2: 9999,
        etvoc: 9999,
        raw_current: 255,
        raw_voltage: 9999,
    };

    let mut counter = 0;
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.blink(500_u16, &mut delay);
        delay.delay_ms(500_u16);

        let data = block!(ccs811.data()).unwrap_or(default);

        counter += 1;
        if counter > 10 {
            counter = 0;

            env = block!(hdc2080.read()).unwrap();
            ccs811
                .set_environment(env.temperature, env.humidity.unwrap_or(0.0))
                .unwrap();
        }

        for i in 0..4 {
            lines[i].clear();
        }
        write!(lines[0], "eCO2: {}", data.eco2).unwrap();
        write!(lines[1], "eTVOC: {}", data.etvoc).unwrap();
        write!(lines[2], "Temp: {:.2}ºC", env.temperature).unwrap();
        write!(lines[3], "Humidity: {:.2}%", env.humidity.unwrap_or(0.0)).unwrap();
        disp.clear();
        for (i, line) in lines.iter().enumerate() {
            Text::new(line, Point::new(0, i as i32 * 16))
                .into_styled(text_style)
                .draw(&mut disp)
                .unwrap();
        }
        disp.flush().unwrap();
    }
}