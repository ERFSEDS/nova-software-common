#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::mem::MaybeUninit;

use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_hal::spi::{Mode, Phase, Polarity};
use hal::pac::USART2;
use ms5611_spi::{Ms5611, Oversampling};
use serde::{Deserialize, Serialize};

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

    delay.delay_ms(2_000u32);

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
    //
    let sck1 = gpioa.pa5.into_alternate();
    let miso1 = gpioa.pa6.into_alternate();
    let mosi1 = gpioa.pa7.into_alternate();

    let baro_cs = gpioc.pc5.into_push_pull_output();
    let gyro_accel_cs = gpiob.pb0.into_push_pull_output();
    let gyro_cs = gpiob.pb1.into_push_pull_output();

    let mut buzzer = gpioc.pc4.into_push_pull_output();

    let mut led_red = gpioc.pc6.into_push_pull_output();
    let mut led_green = gpiob.pb15.into_push_pull_output();
    let mut led_blue = gpiob.pb14.into_push_pull_output();

    led_red.set_low();
    led_green.set_high();
    led_blue.set_high();

    let pin1 = (sck1, miso1, mosi1);

    let spi1 = spi::Spi::new(
        dp.SPI1,
        pin1,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    let sck3 = gpioc.pc10.into_alternate();
    let miso3 = gpioc.pc11.into_alternate();
    let mosi3 = gpioc.pc12.into_alternate();
    let flash_cs = gpiob.pb13.into_push_pull_output();

    let pins3 = (sck3, miso3, mosi3);

    let spi3 = spi::Spi::new(
        dp.SPI3,
        pins3,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    println!();
    println!();
    println!("========================================");
    println!();

    println!("Starting initialization.");

    delay.delay_ms(100u32);

    println!("Initializing flash chip");
    let mut flash = match w25n512gv::new(spi3, flash_cs) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to init flash chip! {:?}", e);
            println!("Starting reset");
            let ptr: *mut u32 = unsafe { core::mem::transmute(0xE000_ED0Cu32) };
            //unsafe { core::ptr::write_volatilen(ptr, 0x05FA_0000) };
            //unsafe { core::ptr::write_volatile(ptr, 0x1 << 2) };
            loop {
                println!("Waiting for reset...");
            }
        }
    };

    flash.modify_configuration_register(|r| {
        *r |= 1 << 4; // Enable ECC
        *r |= 1; // disable HOLD
    });

    // Disable all protections
    flash.modify_protection_register(|r| *r = 0);

    delay.delay_ms(100u32);

    // MODES
    let erase = false;
    let dump_data = true;

    if erase {
        println!("Erasing chip.");
        flash = flash.enable_write().unwrap().erase_all().unwrap();
        delay.delay_ms(100u32);
        flash = flash.enable_write().unwrap().erase_all().unwrap();
        delay.delay_ms(100u32);
        println!("Starting manual erase.");

        led_red.set_low();
        led_green.set_high();
        led_blue.set_high();
        let mut count = 0;
        for i in 0..50_000 {
            flash = flash.enable_write().unwrap().erase_block(i).unwrap();
            if count % 100 == 0 {
                led_red.toggle();
                led_green.toggle();
            }
            count += 1;
        }
        println!("Finished manual erase.");
        let mut buf = [0u8; w25n512gv::PAGE_SIZE_WITH_ECC];
        let mut r = flash.read_sync(64).unwrap();
        r.download_from_buffer_sync(&mut buf).unwrap();
        println!("first data page after flash {:?}", buf);
        loop {
            led_red.set_high();
            led_green.set_high();
            led_blue.set_high();
        }
    }

    println!("Starting initialization.");

    let spi1_bus = shared_bus::BusManagerSimple::new(spi1);

    let mut ms6511 = Ms5611::new(spi1_bus.acquire_spi(), baro_cs, &mut delay)
        .map_err(|_| {
            println!("Barometer failed to initialize.");
        })
        .unwrap();

    println!("Barometer initialized.");

    let mut bmi088_accel = bmi088::Builder::new_accel_spi(spi1_bus.acquire_spi(), gyro_accel_cs);

    if let Err(_) = bmi088_accel.setup(&mut delay) {
        println!("Low-G accelerometer failed to initialize.");
        panic!();
    }

    println!("Low-G accelerometer initialized.");

    let mut bmi088_gyro = bmi088::Builder::new_gyro_spi(spi1_bus.acquire_spi(), gyro_cs);

    if let Err(_) = bmi088_gyro.setup(&mut delay) {
        println!("Gyro failed to initialize.");
        panic!();
    }

    println!("Initialized.");

    println!("Persistent data from last time");

    #[derive(Serialize, Deserialize, Debug)]
    struct GlobalHeader {
        /// The index of the next available block (64 pages)
        block_offset: u32,
        /// The number of times the flight computer has restarted since the flash chip was erased
        num_reboots: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PageHeader {
        /// The index one past the last byte written in this page. This index should be 0x77 if
        /// there is room on the page to help check for errors
        offset: u32,
    }

    //dump_buf(&mut r, &mut page, 64);

    const HEADER_SIZE: usize = 32;

    let mut buf = [0u8; HEADER_SIZE];
    let mut r = flash.read_sync(0).unwrap();
    r.download_from_buffer_sync(&mut buf).unwrap();
    let mut flash = r.finish();

    let mut all_zeroes = true;
    println!("Data {:?}", buf);
    for &val in buf.iter() {
        if val != 0xFF {
            all_zeroes = false;
        }
    }
    /*
    let (mut header, is_initial) = if all_zeroes {
        // First time
        println!("Runnig for the first time");
        (
            GlobalHeader {
                //Start on second block because erasing the start resets us
                block_offset: 1,
                num_reboots: 1,
            },
            true,
        )
    } else {
        println!("Found old header");
        let mut header: GlobalHeader = postcard::from_bytes(&buf).unwrap();
        header.num_reboots += 1;

        (header, false)
    };
    */
    let is_initial = false;
    let mut header = GlobalHeader {
        block_offset: 29,
        num_reboots: 0,
    };

    println!("Found header: {:?}", header);

    if is_initial {
        println!("Entering wait loop");
        let mut largest = 0;
        let mut count = 0;

        led_red.set_high();
        led_green.set_low();
        led_blue.set_low();

        loop {
            if let Ok(sample) = bmi088_accel.get_accel() {
                let total =
                    (sample[0] as i32).abs() + (sample[1] as i32).abs() + (sample[2] as i32).abs();
                if total > largest {
                    largest = total;
                }
                println!("{total} - {largest}");
                if total > 8_000 {
                    //if total > 40_000 {
                    break;
                }
            }
            if count % 1_000 < 200 {
                buzzer.toggle();
            }
            delay.delay_ms(10u32);

            count += 1;
        }
    } else {
        //Dumping data
        if dump_data {
            loop {
                println!("Large amount of data already detected...");
                delay.delay_ms(5_000u32);
                led_red.set_high();
                led_green.set_low();
                led_blue.set_high();

                println!(
                    "Dumping {} blocks, {} pages, {} bytes",
                    header.block_offset,
                    header.block_offset * 64,
                    header.block_offset * 64 * 1024
                );
                let mut buf = [0u8; w25n512gv::PAGE_SIZE_WITH_ECC];
                for block in 1..=header.block_offset {
                    for i in 0..64 {
                        let page_addr = block * 64 + i;
                        println!("Reading {}", page_addr);
                        let mut r = flash.read_sync(page_addr as u16).unwrap();
                        r.download_from_buffer_sync(&mut buf);
                        /*for &byte in &buf {
                            print!("{:X}{:X}", (byte & 0xF) >> 4, byte & 0x0F);
                        }*/

                        let mut dst = [0u8; 4096];
                        let written = base64::encode_config_slice(buf, base64::STANDARD, &mut dst);
                        let s = core::str::from_utf8(&dst[..written]).unwrap();
                        println!("{}", s);
                        println!();

                        flash = r.finish();
                    }
                }
            }
        }
    }

    //disable changing the header so we dont mess with the origional data
    /*let write_header = |flash: w25n512gv::W25n512gvWD<_, _>, header: &[u8]| {
        // We must erase before because we are writing a page that my not be all 1's
        let flash = flash.enable_write().unwrap().erase_block(0).unwrap();
        let r = flash
            .enable_write()
            .unwrap()
            .upload_to_buffer_sync(&header)
            .unwrap();
        let r = r.commit_sync(0).unwrap();
        r.finish()
    };
    */

    postcard::to_slice(&header, &mut buf).unwrap();
    //let mut flash = write_header(flash, &buf);

    struct Buffer<'a> {
        buf: &'a mut [u8],
        offset: usize,
    }

    println!("OK");
    println!(
        "Erasing next block {}, to prevent interference",
        header.block_offset
    );
    let mut flash = flash
        .enable_write()
        .unwrap()
        .erase_block(header.block_offset as u16)
        .unwrap();

    led_red.set_low();
    led_green.set_low();
    led_blue.set_low();

    loop {
        for i in 0..64 {
            //64 pages in a block...
            let mut page = heapless::Vec::<u8, { w25n512gv::PAGE_SIZE_WITH_ECC }>::new();
            page.push(b'N');
            page.push(b'O');
            page.push(b'V');
            page.push(b'A');
            let mut sample_num = 0;
            loop {
                if page.len() > page.capacity() - 8 {
                    //Almost full, flush page
                    break;
                }
                {
                    let sample = ms6511
                        .get_second_order_sample(Oversampling::OS_256, &mut delay)
                        .unwrap();

                    page.push(b'B');
                    page.push(b'B');
                    println!(
                        "Baro  #{}, temp {} pressure {}",
                        sample_num, sample.temperature, sample.pressure
                    );
                    sample_num += 1;

                    write_i32(&mut page, sample.temperature);
                    write_i32(&mut page, sample.pressure);
                    let start = 0i32.max(page.len() as i32 - 16);
                    println!("End of buffer: {:?}", &page[start as usize..]);

                    //add_sample(SampleKind::Pressure, &data)?;
                }

                if let Ok(sample) = bmi088_accel.get_accel() {
                    page.push(b'A');
                    page.push(b'A');

                    write_i16(&mut page, sample[0]);
                    write_i16(&mut page, sample[1]);
                    write_i16(&mut page, sample[2]);

                    println!(
                        "Accel #{}, [{}, {}, {}]",
                        sample_num, sample[0], sample[1], sample[2],
                    );
                    sample_num += 1;

                    let start = 0i32.max(page.len() as i32 - 16);
                    println!("End of buffer: {:?}", &page[start as usize..]);

                    //add_sample(SampleKind::Accel, &data)?;
                }

                if let Ok(sample) = bmi088_gyro.get_gyro() {
                    page.push(b'G');
                    page.push(b'G');

                    write_i16(&mut page, sample[0]);
                    write_i16(&mut page, sample[1]);
                    write_i16(&mut page, sample[2]);

                    println!(
                        "Gyro  #{}, [{}, {}, {}]",
                        sample_num, sample[0], sample[1], sample[2],
                    );
                    sample_num += 1;

                    let start = 0i32.max(page.len() as i32 - 16);
                    println!("End of buffer: {:?}", &page[start as usize..]);

                    //add_sample(SampleKind::Gyro, &data)?;
                }
            }
            let page_addr = header.block_offset * 64 + i;

            let r = flash
                .enable_write()
                .unwrap()
                .upload_to_buffer_sync(&page)
                .unwrap();

            println!();
            println!();
            println!();
            println!("Wrote page!");
            let mut buf = [0u8; 4096];
            let written = base64::encode_config_slice(page, base64::STANDARD, &mut buf);
            let s = core::str::from_utf8(&buf[..written]).unwrap();
            println!("{}", s);

            let r = r.commit_sync(page_addr as u16).unwrap();
            flash = r.finish();
        }
        header.block_offset += 1;
        println!("Filled block. Starting {}", header.block_offset);
        postcard::to_slice(&header, &mut buf).unwrap();
        //flash = write_header(flash, &buf);
    }
}

pub fn write_i16(buf: &mut heapless::Vec<u8, { w25n512gv::PAGE_SIZE_WITH_ECC }>, val: i16) {
    let bytes = val.to_le_bytes();
    buf.push(bytes[0]);
    buf.push(bytes[1]);
}

pub fn write_i32(buf: &mut heapless::Vec<u8, { w25n512gv::PAGE_SIZE_WITH_ECC }>, val: i32) {
    let bytes = val.to_le_bytes();
    buf.push(bytes[0]);
    buf.push(bytes[1]);
    buf.push(bytes[2]);
    buf.push(bytes[3]);
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
