//! State needed by the state machine to execute checks.
//! This state is set as new data values are read, so that when the state machine is executed
//! again, it transparently uses the new data

use novafc_data_format::{BarometerData, Data, Message};

pub struct State {
    pub barometer: Barometer,
}

// We have a big question to ask here. Should we store all these as f32's in proper SI units?
// Because these values arent useful to the state machine when in raw format, however we use the
// raw format to store them for the ground station. It might be better to convert all raw values
// after reading them off the sensors, so that they can be useful to the state machine here, then
// change the data format to use the SI values.
// For now well keep the data format the same and only provide structs here for things that we need
pub struct Barometer {
    /// Altitude from sea level in meters
    pub altitude: f32,

    /// Air temprature in Celsus
    pub temprature: f32,
}

pub struct RawBarometer {
    pub pressure: u32,
    pub temprature: u32,
}

pub trait TimeManager {
    /// Returns the number of ticks since the last call to this function. If more than [`u32::MAX`]
    /// ticks have passed since the last call to `ticks`, this function returns [`u32::MAX`].
    ///
    /// If a trunctaed number of ticks must be returned, implementations should maintain this
    /// state, so that the sum of calling `ticks` thousands of ticks over x wall seconds
    /// is equal to as if `ticks` was called once x seconds since start.
    // u32 is a good middle ground for this value. u64 would allow us to have 1 billion ticks per
    // second and prevent overflow for 584 years. This is nice but overkill. Encoding 1 billion
    // ticks per second as also overkill too.
    // with a u32, we can get 1 million ticks per second and no overflow for over an hour. Given
    // the speed of the microcontroller this is runnign on, this should be fine
    fn ticks(&mut self) -> u32;

    /// Peek at the number of ticks since the last call to [`ticks`] without resetting it.
    fn peek_ticks(&self) -> u32;

    /// Returns the numbe of ticks in a second for this manager
    ///
    /// The tick rate for a given instance of an implementor `Self` must be fixed
    ///
    /// NOTE: Because ticks are 32 bits, the caller should call poll `ticks` at least once every
    /// `u32::MAX/[`tick_rate`] seconds to prevent overflow and ticks from being lost.
    fn tick_rate(&self);
}

pub struct DataBuffer<const N: usize> {
    buf: [u8; N],
    offset: usize,
}

impl<const N: usize> DataBuffer<N> {
    /// Tries to writes a piece of data to this buffer.
    ///
    /// If the buffer is out of space, the message that would have been written
    /// is returned inside Err(..)
    pub fn try_write(&mut self, data: Data, time: &mut impl TimeManager) -> Result<(), Message> {
        self.emit_heartbeats(time)?;
        let ticks = time.ticks();
        let msg = Message {
            // We have called `emit_heartbeats` very recently, so `ticks` is guarnteed to be small
            ticks_since_last_message: ticks.try_into().unwrap(),
            data,
        };
        let buf_len = self.buf.len();
        let buf = self.get_remaining();
        match postcard::to_slice(&msg, buf) {
            Ok(rem) => {
                let new_offset = buf_len - rem.len();
                self.offset = new_offset;
                Ok(())
            }
            Err(err) => {
                match err {
                    postcard::Error::SerializeBufferFull => Err(msg),
                    err => {
                        // This should never happen as we only write plain old data structs
                        // If this were to happen, we would catch it in the simulator
                        panic!("postcard error {}", err)
                    }
                }
            }
        }
    }

    pub fn write(&mut self, data: Data, time: &mut impl TimeManager) {
        self.try_write(data, time).expect("Failed to write message")
    }

    /// Emits a heartbeat message if the number of ticks since the last message does not fit in a
    /// u16.
    ///
    /// This should be used before writing pretty much any other kind of message to ensure that the
    /// reader can properly understand the timing of all messages to prevent tick truncation.
    ///
    /// If the buffer is out of space, the message that would have been written
    /// is returned inside Err(..)
    fn emit_heartbeats(&mut self, time: &mut impl TimeManager) -> Result<(), Message> {
        // Only check about 90% of `u16::MAX` so that if `time.peek_ticks()` is close to u16::MAX,
        // we don't think were ok and then overflow later when we sample `TimeManager::ticks` for real
        const UPPER_BOUND: u32 = u16::MAX as u32 / 9 * 8;
        if time.peek_ticks() > UPPER_BOUND {
            let ticks = time.ticks();
            self.try_write(Data::Heartbeat(ticks), time)?;
        }
        Ok(())
    }

    fn get_remaining(&mut self) -> &mut [u8] {
        &mut self.buf[self.offset..]
    }

    /// Returns all data written to this buffer since the last flush, clearing it for future writes.
    #[must_use]
    pub fn flush(&mut self) -> &[u8] {
        let offset = self.offset;
        self.offset = 0;
        &self.buf[..offset]
    }
}

/// Converts raw sensor data to high level sensor data with SI values
pub trait RawData {
    type Output;

    fn convert(&self) -> Self::Output;

    fn to_data(&self) -> Data;
}

impl RawData for RawBarometer {
    type Output = Barometer;

    fn convert(&self) -> Self::Output {
        todo!()
    }

    fn to_data(&self) -> Data {
        Data::BarometerData(BarometerData {
            temprature: self.temprature,
            pressure: self.pressure,
        })
    }
}
