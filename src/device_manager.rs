use std::collections::HashMap;

use crate::buffer::{SyncDirection, XRTBuffer};
use crate::device::XRTDevice;
use crate::ffi::*;
use crate::kernel::XRTKernel;
use crate::utils::is_null;
use crate::{Error, Result};


pub struct DeviceManager {
    device: XRTDevice,
    kernels: HashMap<String, XRTKernel>,

    // Maybe remove later
    buffers: HashMap<String, XRTBuffer>,
}
