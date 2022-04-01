#![no_std]

use novafc_config_format::reference::{StateTransition, Timeout};
use novafc_config_format::{
    self as config, CheckData, FrozenVec, PyroContinuityCondition, Seconds,
};

use config::reference::{Check, Command, State};
use config::CommandValue;
use static_alloc::Bump;

// Our static allocator
static A: Bump<[u8; 2048]> = Bump::uninit();

fn main() {
    // Flash chip on SPI3
    // USB interface is USART2
    loop {}
}
