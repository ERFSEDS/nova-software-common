#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(adt_const_params)]

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::mem::MaybeUninit;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::{Mode, Phase, Polarity};
use hal::pac::USART2;
use ms5611_spi::Ms5611;

use crate::hal::{pac, prelude::*, spi};
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use w25n512gv::{regs, Addresses, BufferRef, W25n512gv};

static WRITER: Writer = Writer(UnsafeCell::new(MaybeUninit::uninit()));

struct Writer(UnsafeCell<MaybeUninit<hal::serial::Tx<USART2>>>);

unsafe impl Sync for Writer {}
unsafe impl Send for Writer {}

/// # Safety
/// This function must only be called after `WRITER` is initialized
unsafe fn get_writer() -> &'static mut hal::serial::Tx<USART2> {
    unsafe { (*WRITER.0.get()).assume_init_mut() }
}

macro_rules! println {
    () => {{
        let writer = unsafe { get_writer() };
        writeln!(writer).unwrap();
    }};
    ($($arg:tt)*) => {{
        let writer = unsafe { get_writer() };
        writeln!(writer, $($arg)*).unwrap();
    }};
}

macro_rules! print {
    () => {{
        let writer = unsafe { get_writer() };
        write!(writer).unwrap();
    }};
    ($($arg:tt)*) => {{
        let writer = unsafe { get_writer() };
        write!(writer, $($arg)*).unwrap();
    }};
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut delay = dp.TIM1.delay_us(&clocks);

    let tx_pin = gpioa.pa2.into_alternate();

    let serial = dp.USART2.tx(tx_pin, 9600.bps(), &clocks).unwrap();
    let writer = WRITER.0.get();
    unsafe { writer.write(MaybeUninit::new(serial)) };

    // "High-Speed"/H_ SPI for flash chip:
    // SCK PC10
    // MISO PC11
    // MOSI PC12
    //
    // Regular SPI:
    // SCK PA5
    // MISO PA6
    // MOSI PA7
    //
    // CS:
    // FLASH PB13
    // ALTIMETER PC5
    // HIGH_G/ACCEL PB2
    // GYRO ACCEL PB0
    // GYRO PB1
    //

    // All SPI chip selects
    let flash_cs = gpiob.pb13.into_push_pull_output();
    let baro_cs = gpioc.pc5.into_push_pull_output();
    let high_g_accel_cs = gpiob.pb2.into_push_pull_output();
    let gyro_accel_cs = gpiob.pb0.into_push_pull_output();
    let gyro_cs = gpiob.pb1.into_push_pull_output();

    // "High Speed" SPI bus
    let h_sck = gpioc.pc10.into_alternate();
    let h_miso = gpioc.pc11.into_alternate();
    let h_mosi = gpioc.pc12.into_alternate();

    let h_pins = (h_sck, h_miso, h_mosi);

    let h_spi = spi::Spi::new(
        dp.SPI3,
        h_pins,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    // Regular sensor SPI bus
    let sensor_sck = gpioa.pa5.into_alternate();
    let sensor_miso = gpioa.pa6.into_alternate();
    let sensor_mosi = gpioa.pa7.into_alternate();

    let sensor_pins = (sensor_sck, sensor_miso, sensor_mosi);

    let sensor_spi = spi::Spi::new(
        dp.SPI1,
        sensor_pins,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    let sensor_bus = shared_bus::BusManagerSimple::new(sensor_spi);

    println!();
    println!();
    println!("========================================");
    println!();

    println!("Starting initialization.");

    delay.delay_ms(100u32);

    print!("Initializing Barometer ... ");

    let mut ms5611 = Ms5611::new(sensor_bus.acquire_spi(), baro_cs, &mut delay)
        .map_err(|e| {
            println!("Failed: {e:?}");
        })
        .unwrap();
    println!("OK");

    print!("Initializing Low-G Accelerometer ... ");
    let mut bmi088_accel = bmi088::Builder::new_accel_spi(sensor_bus.acquire_spi(), gyro_accel_cs);

    if let Err(e) = bmi088_accel.setup(&mut delay) {
        println!("Failed: {e:?}");
        panic!();
    }
    println!("OK");

    print!("Initializing Gyro ... ");
    let mut bmi088_gyro = bmi088::Builder::new_gyro_spi(sensor_bus.acquire_spi(), gyro_cs);

    if let Err(e) = bmi088_gyro.setup(&mut delay) {
        println!("Failed: {e:?}");
        panic!();
    }
    println!("OK");

    print!("Initializing Flash Chip");
    let flash = w25n512gv::new(h_spi, flash_cs)
        .map_err(|e| {
            println!("Failed: {e:?}");
        })
        .unwrap();

    let (h_spi, flash_cs) = flash.reset(&mut delay);

    let mut flash = w25n512gv::new(h_spi, flash_cs /*, &mut delay*/)
        .map_err(|e| {
            println!("Failed: {e:?}");
        })
        .unwrap();

    flash.modify_configuration_register(|r| {
        r.modify(
            w25n512gv::regs::Configuration::ECC_E::SET + w25n512gv::regs::Configuration::H_DIS::SET,
        )
    });

    // config_val |= 1 << 4; // Enable ECC
    // config_val |= 1; // disable HOLD
    // flash
    //     .write_register(Addresses::CONFIGURATION_REGISTER, config_val)
    //     .unwrap();

    // Disable all protections
    flash.modify_protection_register(|r| r.set(0));

    println!("OK");

    let test_page = 128;

    println!("Persistent data from last time");

    let mut page = [0u8; w25n512gv::PAGE_SIZE_WITH_ECC];
    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 64);
    let flash = r.finish().unwrap();

    println!("Erasing chip...");
    let flash = flash.enable_write().unwrap();
    let flash = flash.erase_all().unwrap().enable_write().unwrap();

    println!("page 0 after erase");

    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 64);
    let flash = r.finish().unwrap().enable_write().unwrap();

    println!("writing first time");
    let mut index: u8 = 0;
    let test_data = [0u8; w25n512gv::PAGE_SIZE_WITH_ECC].map(|_| {
        let before = index;
        index = index.wrapping_add(2);
        before
    });

    let r = flash.upload_to_buffer_sync(&test_data).unwrap();
    let flash = r.commit_sync(test_page).unwrap().finish().unwrap();

    fn dump_buf<SPI, CS, const W: w25n512gv::Writability, const M: w25n512gv::BufMode>(
        r: &mut BufferRef<SPI, CS, W, M>,
        page: &mut [u8; w25n512gv::PAGE_SIZE_WITH_ECC],
        len: usize,
    ) where
        SPI: embedded_hal::blocking::spi::Transfer<u8, Error = stm32f4xx_hal::spi::Error>
            + embedded_hal::blocking::spi::Write<u8, Error = stm32f4xx_hal::spi::Error>,
        CS: OutputPin,
    {
        if let Err(err) = r.download_from_buffer_sync(page) {
            println!("Failed to dump flash buffer!");
            panic!();
        }
        println!("Dumping {} bytes of flash from buffer", len);
        for &byte in page.iter().take(len) {
            print!("{}, ", byte);
        }
        println!();
    }

    println!("after 2 increment write");
    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 16);
    let flash = r.finish().unwrap();

    let mut index: u8 = 0;
    let test_data = [0u8; w25n512gv::PAGE_SIZE_WITH_ECC].map(|_| {
        let before = index;
        index = index.wrapping_add(1);
        before
    });
    delay.delay_us(10u8);

    let flash = flash.enable_write().unwrap();
    let mut flash = flash.erase(test_page).unwrap().enable_write().unwrap();

    println!("writing second time");
    let mut r = flash.upload_to_buffer_sync(&test_data).unwrap();
    let flash = r.commit_sync(test_page).unwrap().finish().unwrap();

    println!("after normal write");

    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 16);
    let flash = r.finish().unwrap();

    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 16);
    let flash = r.finish().unwrap();

    let mut r = flash.read_sync(test_page).unwrap();
    dump_buf(&mut r, &mut page, 16);
    let flash = r.finish().unwrap();

    println!("OK");
    loop {}
}

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
