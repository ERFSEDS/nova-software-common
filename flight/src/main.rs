#![no_std]
#![no_main]

/*
use novafc_config_format::reference::{StateTransition, Timeout};
use novafc_config_format::{
    self as config, CheckData, FrozenVec, PyroContinuityCondition, Seconds,
};

use config::reference::{Check, Command, State};
use config::CommandValue;
use config::{MAX_CHECKS_PER_STATE, MAX_COMMANDS_PER_STATE, MAX_STATES};
use novafc_common::control::Controls;
use novafc_common::data_acquisition::DataWorkspace;
use novafc_common::state_machine::StateMachine;
use static_alloc::Bump;

const STATE_SIZE: usize = core::mem::size_of::<State>() * MAX_STATES;
const CHECK_SIZE: usize = core::mem::size_of::<Check>() * MAX_CHECKS_PER_STATE * MAX_STATES;
const COMMAND_SIZE: usize = core::mem::size_of::<Command>() * MAX_COMMANDS_PER_STATE * MAX_STATES;
const BUMP_SIZE: usize = STATE_SIZE + CHECK_SIZE + COMMAND_SIZE;

// Our static allocator
static A: Bump<[u8; BUMP_SIZE]> = Bump::uninit();
*/

use core::fmt::Write;

use embedded_hal::spi::{Mode, Phase, Polarity};

use crate::hal::{pac, prelude::*, spi};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal as hal;

use ms5611_spi::{Ms5611, Oversampling};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(40.MHz()).freeze();

    let mut delay = dp.TIM1.delay_us(&clocks);

    let tx_pin = gpioa.pa2.into_alternate();

    let mut serial = dp.USART2.tx(tx_pin, 9600.bps(), &clocks).unwrap();

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

    let sck = gpioa.pa5.into_alternate();
    let miso = gpioa.pa6.into_alternate();
    let mosi = gpioa.pa7.into_alternate();
    let baro_cs = gpioc.pc5.into_push_pull_output();
    let accel_cs = gpiob.pb2.into_push_pull_output();

    let pins = (sck, miso, mosi);

    let spi = spi::Spi::new(
        dp.SPI1,
        pins,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        1000.kHz(),
        &clocks,
    );

    writeln!(serial, "Starting initialization.").unwrap();

    let spi_bus = shared_bus::BusManagerSimple::new(spi);

    let mut ms6511 = Ms5611::new(spi_bus.acquire_spi(), baro_cs, &mut delay)
        .map_err(|_| {
            writeln!(serial, "Barometer failed to intialize.").unwrap();
        })
        .unwrap();

    writeln!(serial, "Barometer initialized.").unwrap();

    let mut h3lis331dl = h3lis331dl::H3LIS331DL::new(spi_bus.acquire_spi(), accel_cs)
        .map_err(|e| {
            writeln!(serial, "Accelerometer failed to initialize: {:?}.", e).unwrap();
        })
        .unwrap();

    writeln!(serial, "Accelerometer initialized.").unwrap();

    writeln!(serial, "Initialized.").unwrap();

    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    loop {
        let sample = ms6511
            .get_second_order_sample(Oversampling::OS_256, &mut delay)
            .unwrap();

        h3lis331dl.readAxes(&mut x, &mut y, &mut z).unwrap();

        writeln!(
            serial,
            "Temp: {}, Pressure: {}\nX: {}, Y: {}, Z: {}",
            sample.temperature, sample.pressure, x, y, z
        )
        .unwrap();
    }

    /*
    let increase_data_rate = Command::new(CommandValue::DataRate(16), Seconds::new(4.0));
    let increase_data_rate = &A.leak_box(increase_data_rate).unwrap();
    let launch_commands: FrozenVec<&Command, MAX_COMMANDS_PER_STATE> = FrozenVec::new();
    launch_commands
        .push(increase_data_rate)
        .map_err(|_| ())
        .unwrap();

    let launch = State::new_complete(2, FrozenVec::new(), launch_commands, None);
    let launch = A.leak(launch).map_err(|_| ()).unwrap();

    let safe = State::new_complete(1, FrozenVec::new(), FrozenVec::new(), None);
    let safe = A.leak(safe).map_err(|_| ()).unwrap();

    let poweron_checks: FrozenVec<&Check, MAX_CHECKS_PER_STATE> = FrozenVec::new();
    let continuity_check = Check::new(
        CheckData::Pyro1Continuity(PyroContinuityCondition(true)),
        Some(StateTransition::Transition(launch)),
    );
    let continuity_check = A.leak(continuity_check).map_err(|_| ()).unwrap();

    poweron_checks
        .push(continuity_check)
        .map_err(|_| ())
        .unwrap();

    let poweron = State::new_complete(
        0,
        poweron_checks,
        FrozenVec::new(),
        Some(Timeout::new(
            Seconds::new(3.0),
            StateTransition::Abort(safe),
        )),
    );
    let poweron = A.leak(poweron).map_err(|_| ()).unwrap();

    let data_workspace = DataWorkspace::new();

    let mut controls = Controls::new();

    let mut state_machine = StateMachine::new(poweron, &data_workspace, &mut controls);

    loop {
        state_machine.execute();
    }
    */
}
