#![feature(default_alloc_error_handler)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0};

use cortex_m_rt::entry;
use hal::{pac::USART2, serial::Tx};
use stm32f4xx_hal as hal;

use crate::hal::{gpio::Pull, pac, prelude::*, spi::Spi};

use mfrc522::{comm::eh02::spi::SpiInterface, Mfrc522};

use core::{fmt::Write, panic::PanicInfo};

pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

static mut TX: Option<Tx<USART2>> = None;

#[panic_handler]
fn handle_panic(panic_info: &PanicInfo) -> ! {
    let tx = unsafe { TX.as_mut().unwrap() };
    writeln!(tx, "💥 The program panicked. THIS IS NOT A DRILL!\r").unwrap();
    if let Some(location) = panic_info.location() {
        writeln!(
            tx,
            "📌 Location:  {}:{}:{}\r",
            location.file(),
            location.line(),
            location.column(),
        )
        .unwrap();
    }
    if let Some(message) = panic_info.message() {
        writeln!(tx, "{}\r", message).unwrap();
    }
    loop {}
}

use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static HEAP: CortexMHeap = CortexMHeap::empty();

struct SpiContainer<CS1, CS2, V1, V2, SPI> {
    cs1: CS1,
    cs2: CS2,
    v1: V1,
    v2: V2,
    spi: SPI,
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the LED. On the Nucleo-446RE it's connected to pin PA5.
        let gpioa = dp.GPIOA.split();
        //let mut led = gpioa.pa5.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        let tx_pin = gpioa.pa2;
        let tx = dp.USART2.tx(tx_pin, 9600.bps(), &clocks).unwrap();
        unsafe { TX.replace(tx) };

        let mut cs1 = gpioa.pa4.into_push_pull_output();
        let mut cs2 = gpioa.pa8.into_push_pull_output();
        let v1 = gpioa.pa0.into_push_pull_output();
        let v2 = gpioa.pa1.into_push_pull_output();

        //cs.set_low();
        //cs2.set_high();
        cs1.set_high();
        cs2.set_low();

        let sck = gpioa.pa5.internal_resistor(Pull::Up);
        let miso = gpioa.pa6.internal_resistor(Pull::Down);
        let mosi = gpioa.pa7.internal_resistor(Pull::Down);

        let spi = Spi::new(dp.SPI1, (sck, miso, mosi), MODE_0, 2.MHz(), &clocks);

        let mut container = Some(SpiContainer {
            cs1,
            cs2,
            v1,
            v2,
            spi,
        });

        //let mfrc522_version = mfrc522.version().unwrap();
        //
        //writeln!(unsafe { TX.as_mut().unwrap() }, "{mfrc522_version}");
        let mut tags = Vec::new();

        loop {
            let SpiContainer {
                mut cs1,
                mut cs2,
                mut v1,
                mut v2,
                spi,
            } = container.take().unwrap();

            //cs1.set_high();
            //cs2.set_low();
            cs1.set_low();
            cs2.set_high();
            v1.set_high();
            v2.set_low();

            let itf = SpiInterface::new(spi).with_nss(cs1);
            let mut mfrc522 = Mfrc522::new(itf).init().unwrap();
            //let mfrc522_version = mfrc522.version().unwrap();
            //assert!(mfrc522_version == 0x91 || mfrc522_version == 0x92,);

            if let Ok(atqa) = mfrc522.reqa() {
                if let Ok(uid) = mfrc522.select(&atqa) {
                    let key = [0xD3, 0xF7, 0xD3, 0xF7, 0xD3, 0xF7];
                    let mut data = Vec::new();
                    let mut key_sector = 3 + 4;
                    for block in 4..16 {
                        if block == key_sector {
                            key_sector += 4;
                            continue;
                        }
                        if mfrc522.mf_authenticate(&uid, block, &key).is_ok() {
                            if let Ok(sector) = mfrc522.mf_read(block) {
                                data.extend_from_slice(&sector);
                            }
                        }
                    }

                    let s = String::from_utf8_lossy(&data);

                    if let (Some(start), Some(end)) = (s.find('{'), s.find('}')) {
                        tags.push(s[start..end + 1].to_string());
                    }

                    mfrc522.hlta().unwrap();
                    mfrc522.stop_crypto1().unwrap();
                }
            }
            //mfrc522.hlta().unwrap();

            let conn = mfrc522.release();
            let (spi, mut cs1) = conn.release();
            //cs1.set_low();
            //cs2.set_high();
            cs1.set_high();
            cs2.set_low();
            v1.set_low();
            v2.set_high();
            delay.delay_ms(100_u32);

            let itf = SpiInterface::new(spi).with_nss(cs2);
            let mut mfrc522 = Mfrc522::new(itf).init().unwrap();
            //let mfrc522_version = mfrc522.version().unwrap();
            //assert!(mfrc522_version == 0x91 || mfrc522_version == 0x92);

            if let Ok(atqa) = mfrc522.reqa() {
                if let Ok(uid) = mfrc522.select(&atqa) {
                    let key = [0xD3, 0xF7, 0xD3, 0xF7, 0xD3, 0xF7];
                    let mut data = Vec::new();
                    let mut key_sector = 3 + 4;
                    for block in 4..16 {
                        if block == key_sector {
                            key_sector += 4;
                            continue;
                        }
                        if mfrc522.mf_authenticate(&uid, block, &key).is_ok() {
                            if let Ok(sector) = mfrc522.mf_read(block) {
                                data.extend_from_slice(&sector);
                            }
                        }
                    }

                    let s = String::from_utf8_lossy(&data);

                    if let (Some(start), Some(end)) = (s.find('{'), s.find('}')) {
                        tags.push(s[start..end + 1].to_string());
                    }

                    mfrc522.hlta().unwrap();
                    mfrc522.stop_crypto1().unwrap();
                }
            }
            //mfrc522.hlta().unwrap();

            writeln!(unsafe { TX.as_mut().unwrap() }, "===START===\r").unwrap();
            for tag in &tags {
                writeln!(unsafe { TX.as_mut().unwrap() }, "{tag}\r").unwrap();
            }
            writeln!(unsafe { TX.as_mut().unwrap() }, "===END===\r").unwrap();

            let conn = mfrc522.release();
            let (spi, cs2) = conn.release();

            tags = Vec::new();

            container.replace(SpiContainer {
                cs1,
                cs2,
                v1,
                v2,
                spi,
            });
            delay.delay_ms(100_u32);
        }
    }

    loop {}
}
