#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod control;
pub mod data_acquisition;
pub mod state_machine;
pub mod telemetry;

pub use state_machine::StateMachine;
