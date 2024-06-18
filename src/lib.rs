//! xrt-rs is a wrapper around AMDs XRT C-Bindings, aiming to provide higher level abstraction and error handling.
//!
//! # Example
//! This is roughly how one would use the wrapper to interact with a datacenter FPGA:
//! ```
//! use xrt::device::XRTDevice;
//!
//! let mut device = XRTDevice::from_index(0)
//!     .expect("creating device from index");
//!
//! //TODO think about out how to write a working example
//! //device.load_xclbin("my_xclbin.xclbin")
//! //    .expect("loading xclbin");
//! ```
//!
//! Alternatively, builder-style constructors are also available

#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

pub mod buffer;
pub mod device;
pub mod device_manager;
pub mod error;
pub mod ffi;
pub mod kernel;
pub mod run;
pub mod utils;

pub use error::{Error, Result};
