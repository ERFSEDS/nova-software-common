//! The flight computer -> ground station data logging format.
//!
//! This format is a one to one mapping of the fields that are available on the ground station
//!
//! # Overview
//! Flight computer data can be thought of as a stream of messages.
//! Each message carries a piece of information about the flight computer along with a timestamp
//! of when that message was generated.
//!
//! Because standardizing time on embedded system is hard, this format uses a ticks based system,
//! where the tick rate can be changed inside the format itself. This gives the flight computer
//! lots of flexability to manage its time, while being able to very percisely report exactly when
//! data samples were recorded.
//! Tick 0 is when the flight computer wakes up and there are 1024 ticks per second by default.
//!
//! The data stream generally starts with [`BarometerCalibration`], so that the
//! ground station has the calibration constants it needs, and [`Data::TicksPerSecond`] to
//! establish a custom data rate other than 1024.
//! This is because these actions are done when the flight computer wakes up.
//! The order of messages follows very closley with what the flight computer is doing at any one time,
//! because the current implementation simply reads data, and then immediately records it.
//!
//! # Associated State
//!
//! Any state change on the flight computer (such as a change in calibration constants, or tick
//! rate) that would effect the reconstruction of the data format is always emitted and must be
//! handled.
//! Because of this, decoding implementations must maintain a certain abount of state and update it
//! as new state messages are recieved in order to accuratly reconstruct what happened from the
//! flight computer's point of view.
//!
//! # Assumptions
//!
//! This is the gereral format, however implementations must not make assumptions about the order
//! or quantity of each message type, with the following exceptions:
//! 1. [`Data::BarometerData`] messages will only follow after [`Data::BarometerCalibration`] messages have been
//!    sent before.
//!
//! # Ticks State Example
//!
//! If the first message is a calibration message with [`Message::ticks_since_last_message`] set to 1024,
//! because the default tick rate is 1024, we know that this message was emitted 1 second after
//! flight computer woke up.
//! If the second message is a `TicksPerSecond` message which changes the tick rate to 1,000,000/s,
//! and `ticks_since_last_message` is set to 1024, this change happened 1 second after the calibration
//! message, so 2 seconds total since wakeup.
//! Once this message is processed, all future tick calculations must use the new tick rate.
//! If the third message is a `BarometerData` message recieved 500,000 ticks after the
//! `TicksPerSecond` message, because the new tick rate is 1,000,000 ticks per second,
//! it has been 0.5 seconds since the last message or 2.5 seconds total since wakeup.
//!
//! # Format on the Wire
//!
//! The format of the actual data on the wire is unstable and subject to change, however we plan
//! to use postcard plus serde with these structs until a more efficent bit for bit format can be
//! implemented. Perhaps we could make a crate that automates this process using smaller bit wrapper
//! types U14, U20, u6, etc. to give hints to a proc macro so that enum tags can be packed with data
//! more efficently.

use serde::{Deserialize, Serialize};

/// Calibration values from the barometer's internal memroy,
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

    ///
    HighGAccelerometerData(HighGAccelerometerData),

    /// Indicates how many ticks are in a second.
    /// Ticks are the units used to convey time on the flight computer.
    ///
    /// Before a `TicksPerSecond` message is recieved to indicate otherwise, there are 1024 ticks
    /// in a second.
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
    /// If this is the first message, this tells the number of ticks since MCU startup.
    pub ticks_since_last_message: u16,

    /// The data contained within this message
    pub data: Data,
}

impl Message {
    pub fn new(ticks_since_last_message: u16, data: Data) -> Self {
        Self {
            ticks_since_last_message,
            data,
        }
    }
}

#[test]
fn a() {
    let messages = vec![
        Message::new(
            0,
            Data::BarometerCalibration(BarometerCalibration {
                pressure_sensitivity: 6969,
                pressure_offset: 420,
                temperature_coefficient_ps: 666,
                temperature_coefficient_po: 1427,
                reference_temperature: 1337,
                temperature_coefficient_t: 129,
            }),
        ),
        Message::new(1, Data::TicksPerSecond(1_000_000)),
        Message::new(
            10,
            Data::BarometerData(BarometerData {
                temprature: 76542,
                pressure: 75462,
            }),
        ),
        Message::new(
            746,
            Data::HighGAccelerometerData(HighGAccelerometerData {
                x: -7427,
                y: 32753,
                z: 165,
            }),
        ),
        Message::new(1000, Data::TicksPerSecond(1)),
        Message::new(u16::MAX, Data::Heartbeat(u32::MAX)),
        Message::new(
            1314,
            Data::BarometerData(BarometerData {
                temprature: 76542,
                pressure: 75462,
            }),
        ),
        Message::new(
            0,
            Data::HighGAccelerometerData(HighGAccelerometerData {
                x: -7427,
                y: 32753,
                z: 165,
            }),
        ),
    ];
    let json = serde_json::to_string(&messages).unwrap();
    println!("{}", json);
    panic!();
}
