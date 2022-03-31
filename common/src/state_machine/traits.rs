//! Holds traits that are used by the ground station.

use novafc_config_format::Seconds;

// TODO: switch to #[cfg(ed)] implementation
#[derive(Copy, Clone, Debug)]
pub struct Timestamp(usize);

pub trait GenericTimestamp: std::fmt::Display + std::fmt::Debug + Clone {
    /// Returns a `Timestamp` that represents the instant this function in invoked
    fn now() -> Self;

    /// Returns the number of seconds elapsed between now and this timestamp
    ///
    /// 0 is returned seconds if `Self` is after now
    // TODO: Is is better to panic in this case? What kinds of user code would be messed up if they
    // use this and expect `Self` to always be in the past?
    fn elapsed(&self) -> Seconds {
        self.try_elapsed().unwrap_or_else(|| Seconds::new(0.0))
    }

    /// Returns the number of seconds elapsed between now and this timestamp if timestamp is in the
    /// past.
    ///
    /// If `Self` is in the future, `None` is returned
    fn try_elapsed(&self) -> Option<Seconds>;
}

impl GenericTimestamp for Timestamp {
    fn try_elapsed(&self) -> Option<Seconds> {
        todo!()
    }

    fn now() -> Self {
        todo!()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub trait GpioRead {
    fn read(&self) -> bool;
}

pub trait GpioWrite {
    fn write(&mut self, val: bool);
}

// TODO: switch to #[cfg(ed)] implementation
pub struct Gpio(u16);

impl GpioWrite for Gpio {
    fn write(&mut self, _val: bool) {
        todo!()
    }
}

impl GpioRead for Gpio {
    fn read(&self) -> bool {
        todo!()
    }
}
