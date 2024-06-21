use std::collections::HashMap;

use crate::managed::run::ManagedRun;
use crate::native::device::XRTDevice;
use crate::native::kernel::XRTKernel;
use crate::{Error, Result};

pub struct ManagedDevice {
    pub(crate) device: XRTDevice,
    kernels: HashMap<String, XRTKernel>,
}

impl From<XRTDevice> for ManagedDevice {
    fn from(device: XRTDevice) -> ManagedDevice {
        ManagedDevice {
            device,
            kernels: HashMap::new(),
        }
    }
}

impl ManagedDevice {
    pub fn with_xclbin(mut self, xclbin_path: &str) -> Result<Self> {
        self.device.load_xclbin(xclbin_path)?;
        Ok(self)
    }

    pub fn with_kernel(mut self, kernel_name: &str) -> Result<Self> {
        let kernel = XRTKernel::new(kernel_name, &self.device)?;
        self.kernels.insert(kernel_name.to_string(), kernel);
        Ok(self)
    }

    pub fn run(&self, kernel_name: &str) -> Result<ManagedRun> {
        if let Some(kernel) = self.kernels.get(kernel_name) {
            ManagedRun::new(&self, kernel)
        } else {
            return Err(Error::KernelNotLoadedYetError);
        }
    }
}
