//! The flight computer -> ground station data logging format.
//!
//! This format is a one to one mapping of the fields that are available on the ground station.
//!
//! # Overview
//! Flight computer data can be thought of as a stream of messages.
//! Each message carries a piece of information about the flight computer along with a timestamp
//! of when that message was generated.
//!
//! Because standardizing time on embedded system is hard, this format uses a ticks based system,
//! where the tick rate can be changed inside the format itself. This gives the flight computer
//! lots of flexibility to manage its time, while being able to very percisely report exactly when
//! data samples were recorded.
//!
//! The data stream always starts with a [`Data::TicksPerSecond`] to establish the initial tick
//! rate. The [`Message::ticks_since_last_message`] is ignored in the first message as there is no
//! data rate yet established. Implementations should treat this message as happening at ticks=0.
//! Then it is generally followed with a [`Data::BarometerCalibration`] message, so that the ground
//! station has the calibration constants it needs.
//! This is because these actions are done when the flight computer wakes up.
//! The order of messages follows very closely with what the flight computer is doing at any one time,
//! because the current implementation simply reads data, and then immediately records it.
//!
//! # Associated State
//!
//! Any state change on the flight computer (such as a change in calibration constants, or tick
//! rate) that would effect the reconstruction of the data format is always emitted and must be
//! handled.
//! Because of this, decoding implementations must maintain a certain abount of state and update it
//! as new state messages are recieved in order to accurately reconstruct what happened from the
//! flight computer's point of view.
//!
//! # Assumptions
//!
//! This is the gereral format, however implementations must not make assumptions about the order
//! or quantity of each message type, with the following exceptions:
//! 1. The first message will always be a [`Data::TicksPerSecond`].
//! 2. [`Data::BarometerData`] messages will only follow after one or more
//!    [`Data::BarometerCalibration`] messages have been sent before.
//!
//! # Ticks State Example
//!
//! Consider the following example where the first message is the `TicksPerSecond` message with
//! the value 1024. This establishes the tick rate at 1024 ticks per second.
//!
//! The second message is a calibration message with [`Message::ticks_since_last_message`] set to
//! 2048. Because the current tick rate is 1024, we know that this message was emitted 2 second after
//! the flight computer woke up.
//!
//! The second message is a `TicksPerSecond` message which changes the tick rate to 1,000,000 ticks
//! per second, and `ticks_since_last_message` is set to 512.
//! This change happened `(512 ticks)/(1024 ticks/second) = 0.5 seconds` after the calibration message, so
//! `2 seconds + (512 ticks)/(1024 ticks/second) = 2.5 seconds` total since wakeup.
//! Once this message is processed, all future tick calculations must use the new tick rate.
//!
//! The third message is a `BarometerData` message, recieved 500,000 ticks after the
//! `TicksPerSecond` message.
//! Because the new tick rate is 1,000,000 ticks per second, it has been 0.5 seconds since the
//! last message or 3 seconds total since wakeup.
//!
//! # Format on the Wire
//!
//! The format of the actual data on the wire is unstable and subject to change, however we plan
//! to use Postcard plus Serde with these structs until a more efficent bit for bit format can be
//! implemented. Perhaps we could make a crate that automates this process using smaller bit wrapper
//! types U14, U20, u6, etc. to give hints to a proc macro so that enum tags can be packed with data
//! more efficently.

use serde::{Deserialize, Serialize};

/// Calibration values from the barometer's internal memory,
/// used to convert raw values into unit values
#[derive(Serialize, Deserialize)]
pub struct BarometerCalibration {
    /// Pressure sensitivity | SENS_T1
    pub pressure_sensitivity: u16,
    ///Pressure offset | OFF_T1
    pub pressure_offset: u16,
    /// Temperature coefficient of pressure sensitivity | TCS
    pub temperature_coefficient_ps: u16,
    /// Temperature coefficient of pressure offset | TCO
    pub temperature_coefficient_po: u16,
    /// Reference temperature | T_REF
    pub reference_temperature: u16,
    /// Temperature coefficient of the temperature | TEMPSENS
    pub temperature_coefficient_t: u16,
}

/// Raw data values that come from a single sample of the barometer
#[derive(Serialize, Deserialize)]
pub struct BarometerData {
    pub temprature: u32,
    pub pressure: u32,
}

/// Raw data values that come from a single sample of the barometer
#[derive(Serialize, Deserialize)]
pub struct HighGAccelerometerData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Serialize, Deserialize)]
pub enum Data {
    /// Calibration values from the barometer.
    ///
    /// NOTE: Always sent before `BarometerData` messages
    BarometerCalibration(BarometerCalibration),

    /// Data sample from the barometer
    BarometerData(BarometerData),

    /// Data sample from the high g acceleremoter
    HighGAccelerometerData(HighGAccelerometerData),

    /// Indicates how many ticks are in a second.
    /// Ticks are the units used to convey time on the flight computer.
    ///
    /// NOTE: Always the first message sent
    ///
    /// Each tick `1/current_ticks_per_second` should be added to the reconstructed time, and new
    /// `TicksPerSecond` messages must replace the current `current_ticks_per_second`, so that the
    /// next tick becomes `1/current_ticks_per_second` long.
    TicksPerSecond(u32),

    /// Sent when no other message is sent for a while.
    ///
    /// NOTE: When this message is sent, more computation is needed to determine the _actual_
    /// number of ticks since the last message.
    ///
    /// Add this value to the number of ticks in the message to determine the real number of ticks
    /// since the last message. If this is not done, time will be lost during long periods of no
    /// messages. This is done so that we have extra bits to store more ticks when no messages are
    /// sent for a while, reducing the rate at which we must send messages to avoid overflowing the
    /// small 16 bit number of ticks inside `Message`.
    Heartbeat(u32),
}

/// A message from the flight computer.
/// Many of these messages compose its data stream throughout a flight
#[derive(Serialize, Deserialize)]
pub struct Message {
    /// The number of ticks since the last message in the stream.
    ///
    /// Ignored in the first message
    pub ticks_since_last_message: u16,

    /// The data contained within this message
    pub data: Data,
}
