#![no_std]
#![no_std]
#![no_main]

use core::cell::UnsafeCell;
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::time::Duration;

use embedded_hal::spi::{Mode, Phase, Polarity};
use hal::timer::{Event, Timer};

use mpu9250::Mpu9250;
use ms5611_spi::{Ms5611, Oversampling};

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

    write!(serial, "Starting initialization.").unwrap();

    //let mut mpu9250 = Mpu9250::marg_default(bus2.acquire_spi(), imu_cs, &mut delay).unwrap();

    // let who_am_i = mpu9250.who_am_i().unwrap();
    // panic!("test");
    // let ak8963_who_am_i = mpu9250.ak8963_who_am_i().unwrap();
    // panic!("test");

    // write!(serial, "WHO_aM_I: 0x{:x}", who_am_i);
    // write!(serial, "aK8963_WHO_aM_I: 0x{:x}", ak8963_who_am_i);

    // assert_eq!(who_am_i, 0x71);
    // assert_eq!(ak8963_who_am_i, 0x48);

    //write!(serial, "{:#?}", mpu9250.all().unwrap());

    delay.delay_ms(250u32);

    //write!(serial, "{:#?}", mpu9250.all().unwrap());

    let mut ms6511 = Ms5611::new(bus2.acquire_spi(), baro_cs, &mut delay)
        .map_err(|_| {
            write!(serial, "barometer failed to initialize.").unwrap();
            panic!();
        })
        .unwrap();

    let s = ms6511
        .get_compensated_sample(Oversampling::OS_256, &mut delay)
        .unwrap();
    dump_number(s.pressure as u8);

    let mut h3lis331dl = h3lis331dl::H3LIS331DL::new(bus2.acquire_spi(), high_accel_cs)
        .map_err(|e| {
            write!(serial, "HighG accelerometer failed to initialize: {:?}.", e).unwrap();
            panic!();
        })
        .unwrap();

    let mut x = 0;
    let mut y = 0;
    let mut z = 0;
    h3lis331dl.readAxes(&mut x, &mut y, &mut z).unwrap();
    dump_number(x as u8);

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

struct BadBool(UnsafeCell<bool>);

unsafe impl Send for BadBool {}
unsafe impl Sync for BadBool {}

macro_rules! panic_block {
    ($a:ident, $val:literal) => {
        #[inline(never)]
        fn $a() {
            static mut TEST: BadBool = BadBool(UnsafeCell::new(false));
            // Use this cheese because unconditional panics lead to the function being snipped from
            // the stacktrace.
            // Use unsafe cell and volatile reads to prevent the compiler from being too smart
            unsafe { core::ptr::write_volatile(&mut *TEST.0.get() as *mut bool, true) };

            if unsafe { core::ptr::read_volatile(&mut *TEST.0.get() as *mut bool) } {
                atomic::compiler_fence(Ordering::SeqCst);
                panic!("{}", $val);
            }
        }
    };
}

#[inline]
fn dump_number(val: u8) {
    match val as u16 {
        0x00 => p00(),
        0x01 => p01(),
        0x02 => p02(),
        0x03 => p03(),
        0x04 => p04(),
        0x05 => p05(),
        0x06 => p06(),
        0x07 => p07(),
        0x08 => p08(),
        0x09 => p09(),
        0x0a => p0a(),
        0x0b => p0b(),
        0x0c => p0c(),
        0x0d => p0d(),
        0x0e => p0e(),
        0x0f => p0f(),
        0x10 => p10(),
        0x11 => p11(),
        0x12 => p12(),
        0x13 => p13(),
        0x14 => p14(),
        0x15 => p15(),
        0x16 => p16(),
        0x17 => p17(),
        0x18 => p18(),
        0x19 => p19(),
        0x1a => p1a(),
        0x1b => p1b(),
        0x1c => p1c(),
        0x1d => p1d(),
        0x1e => p1e(),
        0x1f => p1f(),
        0x20 => p20(),
        0x21 => p21(),
        0x22 => p22(),
        0x23 => p23(),
        0x24 => p24(),
        0x25 => p25(),
        0x26 => p26(),
        0x27 => p27(),
        0x28 => p28(),
        0x29 => p29(),
        0x2a => p2a(),
        0x2b => p2b(),
        0x2c => p2c(),
        0x2d => p2d(),
        0x2e => p2e(),
        0x2f => p2f(),
        0x30 => p30(),
        0x31 => p31(),
        0x32 => p32(),
        0x33 => p33(),
        0x34 => p34(),
        0x35 => p35(),
        0x36 => p36(),
        0x37 => p37(),
        0x38 => p38(),
        0x39 => p39(),
        0x3a => p3a(),
        0x3b => p3b(),
        0x3c => p3c(),
        0x3d => p3d(),
        0x3e => p3e(),
        0x3f => p3f(),
        0x40 => p40(),
        0x41 => p41(),
        0x42 => p42(),
        0x43 => p43(),
        0x44 => p44(),
        0x45 => p45(),
        0x46 => p46(),
        0x47 => p47(),
        0x48 => p48(),
        0x49 => p49(),
        0x4a => p4a(),
        0x4b => p4b(),
        0x4c => p4c(),
        0x4d => p4d(),
        0x4e => p4e(),
        0x4f => p4f(),
        0x50 => p50(),
        0x51 => p51(),
        0x52 => p52(),
        0x53 => p53(),
        0x54 => p54(),
        0x55 => p55(),
        0x56 => p56(),
        0x57 => p57(),
        0x58 => p58(),
        0x59 => p59(),
        0x5a => p5a(),
        0x5b => p5b(),
        0x5c => p5c(),
        0x5d => p5d(),
        0x5e => p5e(),
        0x5f => p5f(),
        0x60 => p60(),
        0x61 => p61(),
        0x62 => p62(),
        0x63 => p63(),
        0x64 => p64(),
        0x65 => p65(),
        0x66 => p66(),
        0x67 => p67(),
        0x68 => p68(),
        0x69 => p69(),
        0x6a => p6a(),
        0x6b => p6b(),
        0x6c => p6c(),
        0x6d => p6d(),
        0x6e => p6e(),
        0x6f => p6f(),
        0x70 => p70(),
        0x71 => p71(),
        0x72 => p72(),
        0x73 => p73(),
        0x74 => p74(),
        0x75 => p75(),
        0x76 => p76(),
        0x77 => p77(),
        0x78 => p78(),
        0x79 => p79(),
        0x7a => p7a(),
        0x7b => p7b(),
        0x7c => p7c(),
        0x7d => p7d(),
        0x7e => p7e(),
        0x7f => p7f(),
        0x80 => p80(),
        0x81 => p81(),
        0x82 => p82(),
        0x83 => p83(),
        0x84 => p84(),
        0x85 => p85(),
        0x86 => p86(),
        0x87 => p87(),
        0x88 => p88(),
        0x89 => p89(),
        0x8a => p8a(),
        0x8b => p8b(),
        0x8c => p8c(),
        0x8d => p8d(),
        0x8e => p8e(),
        0x8f => p8f(),
        0x90 => p90(),
        0x91 => p91(),
        0x92 => p92(),
        0x93 => p93(),
        0x94 => p94(),
        0x95 => p95(),
        0x96 => p96(),
        0x97 => p97(),
        0x98 => p98(),
        0x99 => p99(),
        0x9a => p9a(),
        0x9b => p9b(),
        0x9c => p9c(),
        0x9d => p9d(),
        0x9e => p9e(),
        0x9f => p9f(),
        0xa0 => pa0(),
        0xa1 => pa1(),
        0xa2 => pa2(),
        0xa3 => pa3(),
        0xa4 => pa4(),
        0xa5 => pa5(),
        0xa6 => pa6(),
        0xa7 => pa7(),
        0xa8 => pa8(),
        0xa9 => pa9(),
        0xaa => paa(),
        0xab => pab(),
        0xac => pac(),
        0xad => pad(),
        0xae => pae(),
        0xaf => paf(),
        0xb0 => pb0(),
        0xb1 => pb1(),
        0xb2 => pb2(),
        0xb3 => pb3(),
        0xb4 => pb4(),
        0xb5 => pb5(),
        0xb6 => pb6(),
        0xb7 => pb7(),
        0xb8 => pb8(),
        0xb9 => pb9(),
        0xba => pba(),
        0xbb => pbb(),
        0xbc => pbc(),
        0xbd => pbd(),
        0xbe => pbe(),
        0xbf => pbf(),
        0xc0 => pc0(),
        0xc1 => pc1(),
        0xc2 => pc2(),
        0xc3 => pc3(),
        0xc4 => pc4(),
        0xc5 => pc5(),
        0xc6 => pc6(),
        0xc7 => pc7(),
        0xc8 => pc8(),
        0xc9 => pc9(),
        0xca => pca(),
        0xcb => pcb(),
        0xcc => pcc(),
        0xcd => pcd(),
        0xce => pce(),
        0xcf => pcf(),
        0xd0 => pd0(),
        0xd1 => pd1(),
        0xd2 => pd2(),
        0xd3 => pd3(),
        0xd4 => pd4(),
        0xd5 => pd5(),
        0xd6 => pd6(),
        0xd7 => pd7(),
        0xd8 => pd8(),
        0xd9 => pd9(),
        0xda => pda(),
        0xdb => pdb(),
        0xdc => pdc(),
        0xdd => pdd(),
        0xde => pde(),
        0xdf => pdf(),
        0xe0 => pe0(),
        0xe1 => pe1(),
        0xe2 => pe2(),
        0xe3 => pe3(),
        0xe4 => pe4(),
        0xe5 => pe5(),
        0xe6 => pe6(),
        0xe7 => pe7(),
        0xe8 => pe8(),
        0xe9 => pe9(),
        0xea => pea(),
        0xeb => peb(),
        0xec => pec(),
        0xed => ped(),
        0xee => pee(),
        0xef => pef(),
        0xf0 => pf0(),
        0xf1 => pf1(),
        0xf2 => pf2(),
        0xf3 => pf3(),
        0xf4 => pf4(),
        0xf5 => pf5(),
        0xf6 => pf6(),
        0xf7 => pf7(),
        0xf8 => pf8(),
        0xf9 => pf9(),
        0xfa => pfa(),
        0xfb => pfb(),
        0xfc => pfc(),
        0xfd => pfd(),
        0xfe => pfe(),
        0xff => pff(),
        _ => {}
    }
}

panic_block!(p00, 0x00);
panic_block!(p01, 0x01);
panic_block!(p02, 0x02);
panic_block!(p03, 0x03);
panic_block!(p04, 0x04);
panic_block!(p05, 0x05);
panic_block!(p06, 0x06);
panic_block!(p07, 0x07);
panic_block!(p08, 0x08);
panic_block!(p09, 0x09);
panic_block!(p0a, 0x0a);
panic_block!(p0b, 0x0b);
panic_block!(p0c, 0x0c);
panic_block!(p0d, 0x0d);
panic_block!(p0e, 0x0e);
panic_block!(p0f, 0x0f);
panic_block!(p10, 0x10);
panic_block!(p11, 0x11);
panic_block!(p12, 0x12);
panic_block!(p13, 0x13);
panic_block!(p14, 0x14);
panic_block!(p15, 0x15);
panic_block!(p16, 0x16);
panic_block!(p17, 0x17);
panic_block!(p18, 0x18);
panic_block!(p19, 0x19);
panic_block!(p1a, 0x1a);
panic_block!(p1b, 0x1b);
panic_block!(p1c, 0x1c);
panic_block!(p1d, 0x1d);
panic_block!(p1e, 0x1e);
panic_block!(p1f, 0x1f);
panic_block!(p20, 0x20);
panic_block!(p21, 0x21);
panic_block!(p22, 0x22);
panic_block!(p23, 0x23);
panic_block!(p24, 0x24);
panic_block!(p25, 0x25);
panic_block!(p26, 0x26);
panic_block!(p27, 0x27);
panic_block!(p28, 0x28);
panic_block!(p29, 0x29);
panic_block!(p2a, 0x2a);
panic_block!(p2b, 0x2b);
panic_block!(p2c, 0x2c);
panic_block!(p2d, 0x2d);
panic_block!(p2e, 0x2e);
panic_block!(p2f, 0x2f);
panic_block!(p30, 0x30);
panic_block!(p31, 0x31);
panic_block!(p32, 0x32);
panic_block!(p33, 0x33);
panic_block!(p34, 0x34);
panic_block!(p35, 0x35);
panic_block!(p36, 0x36);
panic_block!(p37, 0x37);
panic_block!(p38, 0x38);
panic_block!(p39, 0x39);
panic_block!(p3a, 0x3a);
panic_block!(p3b, 0x3b);
panic_block!(p3c, 0x3c);
panic_block!(p3d, 0x3d);
panic_block!(p3e, 0x3e);
panic_block!(p3f, 0x3f);
panic_block!(p40, 0x40);
panic_block!(p41, 0x41);
panic_block!(p42, 0x42);
panic_block!(p43, 0x43);
panic_block!(p44, 0x44);
panic_block!(p45, 0x45);
panic_block!(p46, 0x46);
panic_block!(p47, 0x47);
panic_block!(p48, 0x48);
panic_block!(p49, 0x49);
panic_block!(p4a, 0x4a);
panic_block!(p4b, 0x4b);
panic_block!(p4c, 0x4c);
panic_block!(p4d, 0x4d);
panic_block!(p4e, 0x4e);
panic_block!(p4f, 0x4f);
panic_block!(p50, 0x50);
panic_block!(p51, 0x51);
panic_block!(p52, 0x52);
panic_block!(p53, 0x53);
panic_block!(p54, 0x54);
panic_block!(p55, 0x55);
panic_block!(p56, 0x56);
panic_block!(p57, 0x57);
panic_block!(p58, 0x58);
panic_block!(p59, 0x59);
panic_block!(p5a, 0x5a);
panic_block!(p5b, 0x5b);
panic_block!(p5c, 0x5c);
panic_block!(p5d, 0x5d);
panic_block!(p5e, 0x5e);
panic_block!(p5f, 0x5f);
panic_block!(p60, 0x60);
panic_block!(p61, 0x61);
panic_block!(p62, 0x62);
panic_block!(p63, 0x63);
panic_block!(p64, 0x64);
panic_block!(p65, 0x65);
panic_block!(p66, 0x66);
panic_block!(p67, 0x67);
panic_block!(p68, 0x68);
panic_block!(p69, 0x69);
panic_block!(p6a, 0x6a);
panic_block!(p6b, 0x6b);
panic_block!(p6c, 0x6c);
panic_block!(p6d, 0x6d);
panic_block!(p6e, 0x6e);
panic_block!(p6f, 0x6f);
panic_block!(p70, 0x70);
panic_block!(p71, 0x71);
panic_block!(p72, 0x72);
panic_block!(p73, 0x73);
panic_block!(p74, 0x74);
panic_block!(p75, 0x75);
panic_block!(p76, 0x76);
panic_block!(p77, 0x77);
panic_block!(p78, 0x78);
panic_block!(p79, 0x79);
panic_block!(p7a, 0x7a);
panic_block!(p7b, 0x7b);
panic_block!(p7c, 0x7c);
panic_block!(p7d, 0x7d);
panic_block!(p7e, 0x7e);
panic_block!(p7f, 0x7f);
panic_block!(p80, 0x80);
panic_block!(p81, 0x81);
panic_block!(p82, 0x82);
panic_block!(p83, 0x83);
panic_block!(p84, 0x84);
panic_block!(p85, 0x85);
panic_block!(p86, 0x86);
panic_block!(p87, 0x87);
panic_block!(p88, 0x88);
panic_block!(p89, 0x89);
panic_block!(p8a, 0x8a);
panic_block!(p8b, 0x8b);
panic_block!(p8c, 0x8c);
panic_block!(p8d, 0x8d);
panic_block!(p8e, 0x8e);
panic_block!(p8f, 0x8f);
panic_block!(p90, 0x90);
panic_block!(p91, 0x91);
panic_block!(p92, 0x92);
panic_block!(p93, 0x93);
panic_block!(p94, 0x94);
panic_block!(p95, 0x95);
panic_block!(p96, 0x96);
panic_block!(p97, 0x97);
panic_block!(p98, 0x98);
panic_block!(p99, 0x99);
panic_block!(p9a, 0x9a);
panic_block!(p9b, 0x9b);
panic_block!(p9c, 0x9c);
panic_block!(p9d, 0x9d);
panic_block!(p9e, 0x9e);
panic_block!(p9f, 0x9f);
panic_block!(pa0, 0xa0);
panic_block!(pa1, 0xa1);
panic_block!(pa2, 0xa2);
panic_block!(pa3, 0xa3);
panic_block!(pa4, 0xa4);
panic_block!(pa5, 0xa5);
panic_block!(pa6, 0xa6);
panic_block!(pa7, 0xa7);
panic_block!(pa8, 0xa8);
panic_block!(pa9, 0xa9);
panic_block!(paa, 0xaa);
panic_block!(pab, 0xab);
panic_block!(pac, 0xac);
panic_block!(pad, 0xad);
panic_block!(pae, 0xae);
panic_block!(paf, 0xaf);
panic_block!(pb0, 0xb0);
panic_block!(pb1, 0xb1);
panic_block!(pb2, 0xb2);
panic_block!(pb3, 0xb3);
panic_block!(pb4, 0xb4);
panic_block!(pb5, 0xb5);
panic_block!(pb6, 0xb6);
panic_block!(pb7, 0xb7);
panic_block!(pb8, 0xb8);
panic_block!(pb9, 0xb9);
panic_block!(pba, 0xba);
panic_block!(pbb, 0xbb);
panic_block!(pbc, 0xbc);
panic_block!(pbd, 0xbd);
panic_block!(pbe, 0xbe);
panic_block!(pbf, 0xbf);
panic_block!(pc0, 0xc0);
panic_block!(pc1, 0xc1);
panic_block!(pc2, 0xc2);
panic_block!(pc3, 0xc3);
panic_block!(pc4, 0xc4);
panic_block!(pc5, 0xc5);
panic_block!(pc6, 0xc6);
panic_block!(pc7, 0xc7);
panic_block!(pc8, 0xc8);
panic_block!(pc9, 0xc9);
panic_block!(pca, 0xca);
panic_block!(pcb, 0xcb);
panic_block!(pcc, 0xcc);
panic_block!(pcd, 0xcd);
panic_block!(pce, 0xce);
panic_block!(pcf, 0xcf);
panic_block!(pd0, 0xd0);
panic_block!(pd1, 0xd1);
panic_block!(pd2, 0xd2);
panic_block!(pd3, 0xd3);
panic_block!(pd4, 0xd4);
panic_block!(pd5, 0xd5);
panic_block!(pd6, 0xd6);
panic_block!(pd7, 0xd7);
panic_block!(pd8, 0xd8);
panic_block!(pd9, 0xd9);
panic_block!(pda, 0xda);
panic_block!(pdb, 0xdb);
panic_block!(pdc, 0xdc);
panic_block!(pdd, 0xdd);
panic_block!(pde, 0xde);
panic_block!(pdf, 0xdf);
panic_block!(pe0, 0xe0);
panic_block!(pe1, 0xe1);
panic_block!(pe2, 0xe2);
panic_block!(pe3, 0xe3);
panic_block!(pe4, 0xe4);
panic_block!(pe5, 0xe5);
panic_block!(pe6, 0xe6);
panic_block!(pe7, 0xe7);
panic_block!(pe8, 0xe8);
panic_block!(pe9, 0xe9);
panic_block!(pea, 0xea);
panic_block!(peb, 0xeb);
panic_block!(pec, 0xec);
panic_block!(ped, 0xed);
panic_block!(pee, 0xee);
panic_block!(pef, 0xef);
panic_block!(pf0, 0xf0);
panic_block!(pf1, 0xf1);
panic_block!(pf2, 0xf2);
panic_block!(pf3, 0xf3);
panic_block!(pf4, 0xf4);
panic_block!(pf5, 0xf5);
panic_block!(pf6, 0xf6);
panic_block!(pf7, 0xf7);
panic_block!(pf8, 0xf8);
panic_block!(pf9, 0xf9);
panic_block!(pfa, 0xfa);
panic_block!(pfb, 0xfb);
panic_block!(pfc, 0xfc);
panic_block!(pfd, 0xfd);
panic_block!(pfe, 0xfe);
panic_block!(pff, 0xff);

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
