//! Optimism-specific constants, types, and helpers.

mod handler_register;

pub use handler_register::{
    bsc_handle_register, collect_system_reward, output, validate_initial_tx_gas, SYSTEM_ADDRESS,
};
