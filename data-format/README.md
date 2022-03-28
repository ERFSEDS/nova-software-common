# novafc-data-format

The flight computer -> ground station data logging format.

This format is a one to one mapping of the fields that are available on the ground station

## Overview
Flight computer data can be thought of as a stream of messages.
Each message carries a piece of information about the flight computer along with a timestamp
of when that message was generated.

Because standardizing time on embedded system is hard, this format uses a ticks based system,
where the tick rate can be changed inside the format itself. This gives the flight computer
lots of flexability to manage its time, while being able to very percisely report exactly when
data samples were recorded.
Tick 0 is when the flight computer wakes up and there are 1024 ticks per second by default.

The data stream generally starts with [`BarometerCalibration`], so that the
ground station has the calibration constants it needs, and [`Data::TicksPerSecond`] to
establish a custom data rate other than 1024.
This is because these actions are done when the flight computer wakes up.
The order of messages follows very closley with what the flight computer is doing at any one time,
because the current implementation simply reads data, and then immediately records it.

## Associated State

Any state change on the flight computer (such as a change in calibration constants, or tick
rate) that would effect the reconstruction of the data format is always emitted and must be
handled.
Because of this, decoding implementations must maintain a certain abount of state and update it
as new state messages are recieved in order to accuratly reconstruct what happened from the
flight computer's point of view.

## Assumptions

This is the gereral format, however implementations must not make assumptions about the order
or quantity of each message type, with the following exceptions:
1. [`Data::BarometerData`] messages will only follow after [`Data::BarometerCalibration`] messages have been
   sent before.

## Ticks State Example

If the first message is a calibration message with [`Message::ticks_since_last_message`] set to 1024,
because the default tick rate is 1024, we know that this message was emitted 1 second after
flight computer woke up.
If the second message is a `TicksPerSecond` message which changes the tick rate to 1,000,000/s,
and `ticks_since_last_message` is set to 1024, this change happened 1 second after the calibration
message, so 2 seconds total since wakeup.
Once this message is processed, all future tick calculations must use the new tick rate.
If the third message is a `BarometerData` message recieved 500,000 ticks after the
`TicksPerSecond` message, because the new tick rate is 1,000,000 ticks per second,
it has been 0.5 seconds since the last message or 2.5 seconds total since wakeup.

## Format on the Wire

The format of the actual data on the wire is unstable and subject to change, however we plan
to use postcard plus serde with these structs until a more efficent bit for bit format can be
implemented. Perhaps we could make a crate that automates this process using smaller bit wrapper
types U14, U20, u6, etc. to give hints to a proc macro so that enum tags can be packed with data
more efficently.

License: MIT
