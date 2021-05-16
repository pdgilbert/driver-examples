//! This plays the final part of Beethoven's ninth symphony given by
//! its MIDI tones using an AD9833 waveform generator / direct digital synthesizer.
//!
//! You can see a video of this running here:
//! https://blog.eldruin.com/ad983x-waveform-generator-dds-driver-in-rust/
//!
//! This example is runs on the STM32F103 "Bluepill" board using  SPI1.
//!
//! ```
//! BP   <-> AD9833  <-> Amplifier
//! GND  <-> VSS     <-> GND
//! 3.3V <-> VDD     <-> VCC
//! PA4  <-> FSYNC
//! PA5  <-> CLK
//! PA7  <-> DAT
//!          OUT     <-> IN
//! ```
//!
//! You will need an amplifier like the PAM8403 or similar and a speaker.
//!
//! Run with:
//! `cargo embed --example ad9833-midi-player-bp`,

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use ad983x::{ad9833_ad9837::Ad9833Ad9837, Ad983x, FrequencyRegister, SpiInterface, MODE};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use libm;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

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

//use stm32f1xx_hal::{delay::Delay, pac, prelude::*, spi::Spi};
#[cfg(feature = "stm32f1xx")]
use stm32f1xx_hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA4, PA5, PA6, PA7},
        Alternate, Floating, Input,
    },
    gpio::{gpioc::PC13, Output, PushPull},
    pac::{CorePeripherals, Peripherals, SPI1},
    prelude::*,
    spi::{Error, Spi, Spi1NoRemap},
};

// would like to use something like   fn setup() -> (impl Ad983x, impl LED, Delay) {
#[cfg(feature = "stm32f1xx")]
fn setup() -> (
    Ad983x<
        SpiInterface<
            Spi<
                SPI1,
                Spi1NoRemap,
                (
                    PA5<Alternate<PushPull>>,
                    PA6<Input<Floating>>,
                    PA7<Alternate<PushPull>>,
                ),
                u8,
            >,
            PA4<Output<PushPull>>,
        >,
        Ad9833Ad9837,
    >,
    impl LED,
    Delay,
) {
    let cp = CorePeripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // SPI1
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        MODE,
        1_u32.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    impl LED for PC13<Output<PushPull>> {
        fn on(&mut self) -> () {
            self.set_low().unwrap()
        }
        fn off(&mut self) -> () {
            self.set_high().unwrap()
        }
    }

    cs.set_high().unwrap();
    let mut synth = Ad983x::new_ad9833(spi, cs);

    (synth, led, delay)
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("AD9833 example");

    let (synth, mut led, mut delay) = setup();

    synth.reset().unwrap();
    synth.enable().unwrap();

    let mut current_register = FrequencyRegister::F0;
    let mut table = MidiTable::default();
    loop {
        // Blink LED 0 to check that everything is actually running.
        // If the LED 0 does not blink, something went wrong.
        led.blink(50_u16, &mut delay);

        let midi_number = table.next().unwrap_or(0);
        let midi_number = f64::from(midi_number);
        let frequency_hz = libm::pow(2.0, (midi_number - 69.0) / 12.0) * 440.0;
        let mclk_hz = 25_000_000.0;
        let synth_value = frequency_hz * f64::from(1 << 28) / mclk_hz;

        // To ensure a smooth transition, set the frequency in the frequency
        // register that is not currently in use, then switch to it.
        let opposite = get_opposite(current_register);
        synth.set_frequency(opposite, synth_value as u32).unwrap();
        synth.select_frequency(opposite).unwrap();
        current_register = opposite;
    }
}

fn get_opposite(register: FrequencyRegister) -> FrequencyRegister {
    match register {
        FrequencyRegister::F0 => FrequencyRegister::F1,
        FrequencyRegister::F1 => FrequencyRegister::F0,
    }
}

#[derive(Debug, Default)]
struct MidiTable {
    position: usize,
    duration_counter: usize,
}

impl Iterator for MidiTable {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut silence = None;
        let (_, note_duration, silence_duration) = Self::NOTES[self.position];
        let total_duration = note_duration + silence_duration;
        let is_in_silence =
            self.duration_counter >= note_duration && self.duration_counter < total_duration;
        if is_in_silence {
            self.duration_counter += 1;
            silence = Some(0);
        } else if self.duration_counter >= total_duration {
            self.position = (self.position + 1) % Self::NOTES.len();
            self.duration_counter = 1;
        } else {
            self.duration_counter += 1;
        }
        let tone = Some(Self::NOTES[self.position].0);
        silence.or(tone)
    }
}

impl MidiTable {
    const NOTES: [(u32, usize, usize); 62] = [
        (76, 4, 1),
        (76, 4, 1),
        (77, 4, 1),
        (79, 4, 1),
        //
        (79, 4, 1),
        (77, 4, 1),
        (76, 4, 1),
        (74, 4, 1),
        //
        (72, 4, 1),
        (72, 4, 1),
        (74, 4, 1),
        (76, 4, 1),
        //
        (76, 4, 4),
        (74, 2, 1),
        (74, 6, 4),
        //
        (76, 4, 1),
        (76, 4, 1),
        (77, 4, 1),
        (79, 4, 1),
        //
        (79, 4, 1),
        (77, 4, 1),
        (76, 4, 1),
        (74, 4, 1),
        //
        (72, 4, 1),
        (72, 4, 1),
        (74, 4, 1),
        (76, 4, 1),
        //
        (74, 4, 4),
        (72, 2, 1),
        (72, 6, 4),
        //
        (74, 4, 1),
        (74, 4, 1),
        (76, 4, 1),
        (72, 4, 1),
        //
        (74, 4, 1),
        (76, 2, 1),
        (77, 2, 1),
        (76, 4, 1),
        (72, 4, 1),
        //
        (74, 4, 1),
        (76, 2, 1),
        (77, 2, 1),
        (76, 4, 1),
        (74, 4, 1),
        //
        (72, 4, 1),
        (74, 4, 1),
        (67, 6, 2),
        //
        (76, 4, 1),
        (76, 4, 1),
        (77, 4, 1),
        (79, 4, 1),
        //
        (79, 4, 1),
        (77, 4, 1),
        (76, 4, 1),
        (74, 4, 1),
        //
        (72, 4, 1),
        (72, 4, 1),
        (74, 4, 1),
        (76, 4, 1),
        //
        (74, 6, 2),
        (72, 2, 1),
        (72, 6, 10),
    ];
}