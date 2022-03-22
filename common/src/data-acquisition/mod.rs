#![cfg_attr(not(feature = "std"), no_std)]

use std::time::{Duration, Instant};

use data_format::{CheckKind, ObjectState};

pub struct DataWorkspace {
    altitude: SimulatedDataObject,
    pyro1: SimulatedDataObject,
    pyro2: SimulatedDataObject,
    pyro3: SimulatedDataObject,
}

impl DataWorkspace {
    pub fn new() -> Self {
        let now = Instant::now();

        let altitude = SimulatedDataObject::DurationBased(DurationBased::new(
            ObjectState::Flag(false),
            ObjectState::Flag(true),
            now + Duration::from_secs(2),
        ));

        let pyro1 = SimulatedDataObject::DurationBased(DurationBased::new(
            ObjectState::Flag(false),
            ObjectState::Flag(true),
            now + Duration::from_secs(2),
        ));
        let pyro2 = SimulatedDataObject::DurationBased(DurationBased::new(
            ObjectState::Flag(false),
            ObjectState::Flag(true),
            now + Duration::from_secs(2),
        ));
        let pyro3 = SimulatedDataObject::DurationBased(DurationBased::new(
            ObjectState::Flag(false),
            ObjectState::Flag(true),
            now + Duration::from_secs(2),
        ));

        Self {
            altitude,
            pyro1,
            pyro2,
            pyro3,
        }
    }

    pub fn get_object(&self, object: CheckKind) -> ObjectState {
        match object {
            CheckKind::Altitude => self.altitude.read(),
            CheckKind::ApogeeFlag => {
                let _alt = self.altitude.read();
                // Need more state here to know when we have passed apogee
                unimplemented!()
                //ObjectState::Flag(past_apogee)
            }
            CheckKind::Pyro1Continuity => self.pyro1.read(),
            CheckKind::Pyro2Continuity => self.pyro2.read(),
            CheckKind::Pyro3Continuity => self.pyro3.read(),
        }
    }
}

/// A struct that stores a GPIO pin that can be read at any time
struct Gpio {
    pin: u16,
}

impl Gpio {
    fn new(pin: u16) -> Self {
        Self { pin }
    }

    fn read(&self) -> ObjectState {
        unimplemented!();
    }
}

pub trait DataObject {
    fn read(&self) -> ObjectState;
}

/// Represents any source of an ObjectState
enum SimulatedDataObject {
    Gpio(Gpio),
    DurationBased(DurationBased),
}

impl DataObject for SimulatedDataObject {
    fn read(&self) -> ObjectState {
        match self {
            Self::Gpio(gpio) => gpio.read(),
            Self::DurationBased(db) => db.read(),
        }
    }
}

/// Used to simulate a change in values at a particular point in time for testing
struct DurationBased {
    /// The initial value of this state, will be returned in [`DurationBased::read`]
    /// if before `transition_at`
    pub initial: ObjectState,

    /// The final value of this state, will be returned in [`DurationBased::read`]
    /// if after `transition_at`
    // Would be nice if we could call this `final`, but final is a reserved keyword :(
    pub eventual: ObjectState,

    /// The instant in time to transition between `initial` and `eventual`
    pub transition_at: Instant,
}

impl DurationBased {
    pub fn new(initial: ObjectState, eventual: ObjectState, transition_at: Instant) -> Self {
        Self {
            initial,
            eventual,
            transition_at,
        }
    }

    fn read(&self) -> ObjectState {
        let now = Instant::now();
        if self.transition_at > now {
            self.eventual
        } else {
            self.initial
        }
    }
}
