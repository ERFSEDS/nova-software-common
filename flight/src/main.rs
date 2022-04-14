#![no_std]
#![no_std]
#![no_main]

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::time::Duration;

use embedded_hal::spi::{Mode, Phase, Polarity};
use hal::pac::USART2;
use hal::timer::{Event, Timer};
use mpu9250::Mpu9250;

use crate::hal::{pac, prelude::*, spi};
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(100.MHz()).freeze();

    let mut delay = dp.TIM1.delay_us(&clocks);

    let mut blue_led = gpiob.pb7.into_push_pull_output();
    let mut green_led = gpiob.pb8.into_push_pull_output();
    let mut red_led = gpiob.pb9.into_push_pull_output();

    let tx_pin = gpioa.pa11.into_alternate();

    let mut serial = dp.USART6.tx(tx_pin, 9600.bps(), &clocks).unwrap();
    write!(serial, "Test").unwrap();

    let sck = gpiob.pb13.into_alternate();
    let miso = gpiob.pb14.into_alternate();
    let mosi = gpiob.pb15.into_alternate();
    let baro_cs = gpiob.pb12.into_push_pull_output();
    let imu_cs = gpioa.pa8.into_push_pull_output();
    let high_accel_cs = gpiob.pb10.into_push_pull_output();

    let spi2_pins = (sck, miso, mosi);

    let spi2 = spi::Spi::new(
        dp.SPI2,
        spi2_pins,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    let bus2 = shared_bus::BusManagerSimple::new(spi2);
    let spi2 = bus2.acquire_spi();

    write!(serial, "Starting initialization.").unwrap();

    let mut mpu9250 = Mpu9250::marg_default(spi2, imu_cs, &mut delay).unwrap();


    let who_am_i = mpu9250.who_am_i().unwrap();
    panic!("test");
    let ak8963_who_am_i = mpu9250.ak8963_who_am_i().unwrap();
    panic!("test");

    write!(serial, "WHO_AM_I: 0x{:x}", who_am_i);
    write!(serial, "AK8963_WHO_AM_I: 0x{:x}", ak8963_who_am_i);

    assert_eq!(who_am_i, 0x71);
    assert_eq!(ak8963_who_am_i, 0x48);

    //write!(serial, "{:#?}", mpu9250.all().unwrap());

    delay.delay_ms(250u32);

    //write!(serial, "{:#?}", mpu9250.all().unwrap());

    loop {
        blue_led.set_high();
        red_led.set_high();
        green_led.set_low();
        delay.delay_ms(250u32);

        blue_led.set_low();
        red_led.set_low();
        green_led.set_high();
        delay.delay_ms(250u32);

        blue_led.set_low();
        red_led.set_low();
        green_led.set_low();

        delay.delay_ms(250u32);
    }
}

fn dump_number(val: u8) -> ! {
    match val {
        0x00 => panic!(),
        0x01 => panic!(),
        0x02 => panic!(),
        0x03 => panic!(),
        0x04 => panic!(),
        0x05 => panic!(),
        0x06 => panic!(),
        0x07 => panic!(),
        0x08 => panic!(),
        0x09 => panic!(),
        0x0A => panic!(),
        0x0B => panic!(),
        0x0C => panic!(),
        0x0D => panic!(),
        0x0E => panic!(),
        0x0F => panic!(),
        0x10 => panic!(),
        0x11 => panic!(),
        0x12 => panic!(),
        0x13 => panic!(),
        0x14 => panic!(),
        0x15 => panic!(),
        0x16 => panic!(),
        0x17 => panic!(),
        0x18 => panic!(),
        0x19 => panic!(),
        0x1A => panic!(),
        0x1B => panic!(),
        0x1C => panic!(),
        0x1D => panic!(),
        0x1E => panic!(),
        0x1F => panic!(),
        0x20 => panic!(),
        0x21 => panic!(),
        0x22 => panic!(),
        0x23 => panic!(),
        0x24 => panic!(),
        0x25 => panic!(),
        0x26 => panic!(),
        0x27 => panic!(),
        0x28 => panic!(),
        0x29 => panic!(),
        0x2A => panic!(),
        0x2B => panic!(),
        0x2C => panic!(),
        0x2D => panic!(),
        0x2E => panic!(),
        0x2F => panic!(),
        0x30 => panic!(),
        0x31 => panic!(),
        0x32 => panic!(),
        0x33 => panic!(),
        0x34 => panic!(),
        0x35 => panic!(),
        0x36 => panic!(),
        0x37 => panic!(),
        0x38 => panic!(),
        0x39 => panic!(),
        0x3A => panic!(),
        0x3B => panic!(),
        0x3C => panic!(),
        0x3D => panic!(),
        0x3E => panic!(),
        0x3F => panic!(),
        0x40 => panic!(),
        0x41 => panic!(),
        0x42 => panic!(),
        0x43 => panic!(),
        0x44 => panic!(),
        0x45 => panic!(),
        0x46 => panic!(),
        0x47 => panic!(),
        0x48 => panic!(),
        0x49 => panic!(),
        0x4A => panic!(),
        0x4B => panic!(),
        0x4C => panic!(),
        0x4D => panic!(),
        0x4E => panic!(),
        0x4F => panic!(),
        0x50 => panic!(),
        0x51 => panic!(),
        0x52 => panic!(),
        0x53 => panic!(),
        0x54 => panic!(),
        0x55 => panic!(),
        0x56 => panic!(),
        0x57 => panic!(),
        0x58 => panic!(),
        0x59 => panic!(),
        0x5A => panic!(),
        0x5B => panic!(),
        0x5C => panic!(),
        0x5D => panic!(),
        0x5E => panic!(),
        0x5F => panic!(),
        0x60 => panic!(),
        0x61 => panic!(),
        0x62 => panic!(),
        0x63 => panic!(),
        0x64 => panic!(),
        0x65 => panic!(),
        0x66 => panic!(),
        0x67 => panic!(),
        0x68 => panic!(),
        0x69 => panic!(),
        0x6A => panic!(),
        0x6B => panic!(),
        0x6C => panic!(),
        0x6D => panic!(),
        0x6E => panic!(),
        0x6F => panic!(),
        0x70 => panic!(),
        0x71 => panic!(),
        0x72 => panic!(),
        0x73 => panic!(),
        0x74 => panic!(),
        0x75 => panic!(),
        0x76 => panic!(),
        0x77 => panic!(),
        0x78 => panic!(),
        0x79 => panic!(),
        0x7A => panic!(),
        0x7B => panic!(),
        0x7C => panic!(),
        0x7D => panic!(),
        0x7E => panic!(),
        0x7F => panic!(),
        0x80 => panic!(),
        0x81 => panic!(),
        0x82 => panic!(),
        0x83 => panic!(),
        0x84 => panic!(),
        0x85 => panic!(),
        0x86 => panic!(),
        0x87 => panic!(),
        0x88 => panic!(),
        0x89 => panic!(),
        0x8A => panic!(),
        0x8B => panic!(),
        0x8C => panic!(),
        0x8D => panic!(),
        0x8E => panic!(),
        0x8F => panic!(),
        0x90 => panic!(),
        0x91 => panic!(),
        0x92 => panic!(),
        0x93 => panic!(),
        0x94 => panic!(),
        0x95 => panic!(),
        0x96 => panic!(),
        0x97 => panic!(),
        0x98 => panic!(),
        0x99 => panic!(),
        0x9A => panic!(),
        0x9B => panic!(),
        0x9C => panic!(),
        0x9D => panic!(),
        0x9E => panic!(),
        0x9F => panic!(),
        0xA0 => panic!(),
        0xA1 => panic!(),
        0xA2 => panic!(),
        0xA3 => panic!(),
        0xA4 => panic!(),
        0xA5 => panic!(),
        0xA6 => panic!(),
        0xA7 => panic!(),
        0xA8 => panic!(),
        0xA9 => panic!(),
        0xAA => panic!(),
        0xAB => panic!(),
        0xAC => panic!(),
        0xAD => panic!(),
        0xAE => panic!(),
        0xAF => panic!(),
        0xB0 => panic!(),
        0xB1 => panic!(),
        0xB2 => panic!(),
        0xB3 => panic!(),
        0xB4 => panic!(),
        0xB5 => panic!(),
        0xB6 => panic!(),
        0xB7 => panic!(),
        0xB8 => panic!(),
        0xB9 => panic!(),
        0xBA => panic!(),
        0xBB => panic!(),
        0xBC => panic!(),
        0xBD => panic!(),
        0xBE => panic!(),
        0xBF => panic!(),
        0xC0 => panic!(),
        0xC1 => panic!(),
        0xC2 => panic!(),
        0xC3 => panic!(),
        0xC4 => panic!(),
        0xC5 => panic!(),
        0xC6 => panic!(),
        0xC7 => panic!(),
        0xC8 => panic!(),
        0xC9 => panic!(),
        0xCA => panic!(),
        0xCB => panic!(),
        0xCC => panic!(),
        0xCD => panic!(),
        0xCE => panic!(),
        0xCF => panic!(),
        0xD0 => panic!(),
        0xD1 => panic!(),
        0xD2 => panic!(),
        0xD3 => panic!(),
        0xD4 => panic!(),
        0xD5 => panic!(),
        0xD6 => panic!(),
        0xD7 => panic!(),
        0xD8 => panic!(),
        0xD9 => panic!(),
        0xDA => panic!(),
        0xDB => panic!(),
        0xDC => panic!(),
        0xDD => panic!(),
        0xDE => panic!(),
        0xDF => panic!(),
        0xE0 => panic!(),
        0xE1 => panic!(),
        0xE2 => panic!(),
        0xE3 => panic!(),
        0xE4 => panic!(),
        0xE5 => panic!(),
        0xE6 => panic!(),
        0xE7 => panic!(),
        0xE8 => panic!(),
        0xE9 => panic!(),
        0xEA => panic!(),
        0xEB => panic!(),
        0xEC => panic!(),
        0xED => panic!(),
        0xEE => panic!(),
        0xEF => panic!(),
        0xF0 => panic!(),
        0xF1 => panic!(),
        0xF2 => panic!(),
        0xF3 => panic!(),
        0xF4 => panic!(),
        0xF5 => panic!(),
        0xF6 => panic!(),
        0xF7 => panic!(),
        0xF8 => panic!(),
        0xF9 => panic!(),
        0xFA => panic!(),
        0xFB => panic!(),
        0xFC => panic!(),
        0xFD => panic!(),
        0xFE => panic!(),
        0xFF => panic!(),
    }
}

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
