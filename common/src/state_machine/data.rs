//! State needed by the state machine to execute checks.
//! This state is set as new data values are read, so that when the state machine is executed
//! again, it transparently uses the new data

use novafc_data_format::{BarometerData, Data, Message};

pub struct Samples {
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
    fn tick_rate(&self) -> u32;
}

pub struct Buffer<'b> {
    buf: &'b mut [u8],
    offset: usize,
}

/// A double buffering system used to prevent loss of writes
pub struct BufferedBuffer<'b, 'e> {
    buffer: Buffer<'b>,
    extra: &'e mut [u8],
}

impl<'b, 'e> BufferedBuffer<'b, 'e> {
    pub fn new(buf: &'b mut [u8], extra: &'e mut [u8]) -> Self {
        Self {
            buffer: Buffer::new(buf),
            extra,
        }
    }

    /// Writes a data sample to the buffer system.
    ///
    /// When [`FlushRequired::Yes`] is returned, the user must flush the content obtained using
    /// [`FlushInfo::buffer`] to the final source of the data
    pub fn write<'s>(
        &'s mut self,
        data: Data,
        time: &mut impl TimeManager,
    ) -> FlushRequired<'s, 'b, 'e> {
        match self.buffer.try_write(data, time) {
            Ok(_) => FlushRequired::No, // all good
            Err(data) => {
                // The buffer is too full!
                // Serialize to `remaining` then fully fill `buf`
                let mut extra_buf = Buffer::new(&mut self.extra);
                match extra_buf.try_write(data, time) {
                    Ok(count_in_extra) => {
                        // Writes `remaining` bytes to `buffer`
                        let count_in_buffer =
                            self.buffer.write_bytes(&self.extra[..count_in_extra]);

                        // Store the required info here so that on drop we copy the rest
                        // We already copied `extra[remaining..]`
                        // We want to copy from
                        FlushRequired::Yes(FlushInfo {
                            buffer: self,
                            extra_offset: count_in_buffer,
                            extra_len: count_in_extra - count_in_buffer,
                        })
                    }
                    Err(_) => panic!(),
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Holds information a user needs to flush a [`BufferedBuffer`]
pub struct FlushInfo<'s, 'b, 'e> {
    buffer: &'s mut BufferedBuffer<'b, 'e>,

    /// The index of the first byte inside `extra` that needs to be copied to the beginning of
    /// `buffer.buf`, once the main data is
    extra_offset: usize,

    /// How many bytes need to be copied to the beginning of `buffer.buf` from `extra`, once the main data is
    /// flushed
    extra_len: usize,
}

pub enum FlushRequired<'s, 'b, 'e> {
    Yes(FlushInfo<'s, 'b, 'e>),
    No,
}

impl<'s, 'b, 'e> Drop for FlushInfo<'s, 'b, 'e> {
    fn drop(&mut self) {
        let to_write = &self.buffer.extra[self.extra_offset..self.extra_offset + self.extra_len];
        self.buffer.buffer.clear();
        self.buffer.buffer.write_bytes(to_write);
    }
}

impl<'b> Buffer<'b> {
    /// Creates a new, empty buffer with storage backed by `buf`
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    /// Tries to writes a piece of data to this buffer, retuning the number of bytes written
    ///
    /// If the buffer is out of space, the message that would have been written
    /// is returned inside Err(..)
    pub fn try_write(&mut self, data: Data, time: &mut impl TimeManager) -> Result<usize, Data> {
        self.emit_heartbeats(time).map_err(|_| data.clone())?;
        let ticks = time.ticks();
        let msg = Message {
            // We have called `emit_heartbeats` very recently, so `ticks` is guarnteed to be small
            ticks_since_last_message: ticks.try_into().unwrap(),
            data,
        };
        let capacity = self.buf.len();
        let unused = &mut self.buf[self.offset..];
        let unused_len = unused.len();
        match postcard::to_slice(&msg, unused) {
            Ok(rem) => {
                self.offset = capacity - rem.len();
                Ok(unused_len - rem.len())
            }
            Err(err) => {
                match err {
                    postcard::Error::SerializeBufferFull => Err(msg.data),
                    err => {
                        // This should never happen as we only write plain old data structs
                        // If this were to happen, we would catch it in the simulator
                        panic!("postcard error {}", err)
                    }
                }
            }
        }
    }

    /// Emits a heartbeat message if the number of ticks since the last message does not fit in a
    /// u16.
    ///
    /// This should be used before writing pretty much any other kind of message to ensure that the
    /// reader can properly understand the timing of all messages to prevent tick truncation.
    ///
    /// If the buffer is out of space, Err(()) is returned
    fn emit_heartbeats(&mut self, time: &mut impl TimeManager) -> Result<(), ()> {
        // Only check about 90% of `u16::MAX` so that if `time.peek_ticks()` is close to u16::MAX,
        // we don't think were ok and then overflow later when we sample `TimeManager::ticks` for real
        const UPPER_BOUND: u32 = u16::MAX as u32 / 9 * 8;
        if time.peek_ticks() > UPPER_BOUND {
            let ticks = time.ticks();
            self.try_write(Data::Heartbeat(ticks), time).map_err(|_| ());
        }
        Ok(())
    }

    /// Clears the data in this buffer
    #[inline]
    pub fn clear(&mut self) {
        self.offset = 0;
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.offset
    }

    /// Writes bytes from `src` into self, returning the number of bytes written.
    ///
    /// If `src.len()` is larger than `self.bytes_remaining()`, then the end of `src` is ignored
    /// and `self.bytes_remaining()` bytes are written
    pub fn write_bytes(&mut self, src: &[u8]) -> usize {
        let len = src.len().min(self.remaining());
        self.buf[self.offset..self.offset + len].copy_from_slice(&src[..len]);
        self.offset += len;
        len
    }

    /// Manually returns all data written to this buffer since the last flush, clearing it for future writes.
    #[must_use]
    #[inline]
    pub fn flush(&mut self) -> &[u8] {
        let offset = self.offset;
        self.clear();
        &self.buf[..offset]
    }

    /// Returns all data written to this buffer since the last flush.
    #[must_use]
    #[inline]
    pub fn data(&mut self) -> &[u8] {
        &self.buf[..self.offset]
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, BufferedBuffer};
    #[test]
    fn basic_buffer() {
        let mut buf = [0u8; 16];
        let mut buffer = Buffer::new(&mut buf);
        assert_eq!(buffer.remaining(), 16);
        assert_eq!(buffer.write_bytes(&[0, 1, 2, 3]), 4);
        assert_eq!(buffer.data(), &[0, 1, 2, 3]);
        assert_eq!(buffer.remaining(), 12);

        assert_eq!(buffer.write_bytes(&[5, 6, 7, 8, 8, 8]), 6);
        assert_eq!(buffer.data(), &[0, 1, 2, 3, 5, 6, 7, 8, 8, 8]);
        assert_eq!(buffer.remaining(), 6);

        assert_eq!(buffer.write_bytes(&[9, 10, 11, 12, 13, 14, 15, 16]), 6);
        assert_eq!(buffer.remaining(), 0);
        assert_eq!(buffer.write_bytes(&[0, 1, 2, 3]), 0);
        assert_eq!(buffer.remaining(), 0);
        assert_eq!(buffer.write_bytes(&[0, 1, 2, 3]), 0);
        assert_eq!(buffer.remaining(), 0);
        assert_eq!(buffer.write_bytes(&[0, 1, 2, 3]), 0);
        assert_eq!(buffer.remaining(), 0);

        assert_eq!(
            buffer.data(),
            &[0, 1, 2, 3, 5, 6, 7, 8, 8, 8, 9, 10, 11, 12, 13, 14]
        );
        assert_eq!(
            buffer.flush(),
            &[0, 1, 2, 3, 5, 6, 7, 8, 8, 8, 9, 10, 11, 12, 13, 14]
        );
        assert_eq!(buffer.flush(), &[]);
        assert_eq!(buffer.data(), &[]);
        assert_eq!(buffer.flush(), &[]);
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
            //TODO: FIXME
            temprature: self.temprature,
            pressure: self.pressure,
        })
    }
}
