#![cfg_attr(not(feature = "std"), no_std)]
#[deny(unsafe_op_in_unsafe_fn)]

pub mod telemetry;
pub mod control;
pub mod state_machine;
pub mod config_format;
pub mod data_acquisition;
