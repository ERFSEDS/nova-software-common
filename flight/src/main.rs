#![no_std]

use novafc_common::state_machine::state::{self, RawData};
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

fn main() {
    let increase_data_rate = Command::new(CommandValue::DataRate(16), Seconds::new(4.0));
    let increase_data_rate = &A.leak_box(increase_data_rate).unwrap();
    let mut launch_commands: FrozenVec<&Command, MAX_COMMANDS_PER_STATE> = FrozenVec::new();
    launch_commands
        .push(&increase_data_rate)
        .map_err(|_| ())
        .unwrap();

    let launch = State::new_complete(2, FrozenVec::new(), launch_commands, None);
    let launch = A.leak(launch).map_err(|_| ()).unwrap();

    let safe = State::new_complete(1, FrozenVec::new(), FrozenVec::new(), None);
    let safe = A.leak(safe).map_err(|_| ()).unwrap();

    let mut poweron_checks: FrozenVec<&Check, MAX_CHECKS_PER_STATE> = FrozenVec::new();
    let continuity_check = Check::new(
        CheckData::Pyro1Continuity(PyroContinuityCondition(true)),
        Some(StateTransition::Transition(launch)),
    );
    let continuity_check = A.leak(continuity_check).map_err(|_| ()).unwrap();

    poweron_checks
        .push(&continuity_check)
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

    let mut state_machine = StateMachine::new(&poweron);

    let mut buf = DataBuffer {};
    let state = state::State {
        barometer: state::Barometer {
            altitude: 0.0,
            temprature: 0.0,
        },
    };

    loop {
        state_machine.execute(&state);
        data::aquire(&mut buf, &mut state);
    }
}

pub trait GenericSensor {
    type Output: RawData;
    fn read(&self) -> Option<Self::Output>;
}

fn do_data(sensor: &impl GenericSensor) {
    if let Some(raw) = sensor.read() {
        let barometer_data = raw.convert();
        //Write the data
        buf.write(raw_barometer_data);
    }
}

mod data {
    use super::*;
    use state::RawData;

    // Reads data, writes it to buf for storage onto the flash chip/sd card, and updates
    // internal data registers for the state machine
    pub fn aquire(buf: &mut DataBuffer, state: &mut State, barometer: &impl GenericSensor) {
        if let Some(raw_barometer_data) = barometer.read() {
            let barometer_data = raw_barometer_data.convert();
            //Write the data
            buf.write(raw_barometer_data);
        }
    }
}
