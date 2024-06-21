use std::collections::HashMap;

use crate::buffer::XRTBuffer;
use crate::device::XRTDevice;
use crate::kernel::XRTKernel;
use crate::run::XRTRun;
use crate::{Error, Result};

pub trait HardwareDatatype {}

impl HardwareDatatype for u32 {}
impl HardwareDatatype for i32 {}
impl HardwareDatatype for u64 {}
impl HardwareDatatype for i64 {}
impl HardwareDatatype for f32 {}
impl HardwareDatatype for f64 {}

pub enum ArgumentType<'a, T: HardwareDatatype> {
    Scalar(T),
    Buffer(&'a [T]),
}

// contains a run and its corresponding buffers
pub struct ManagedRun<'a> {
    run: XRTRun,
    buffers: HashMap<i32, XRTBuffer>,
    device: &'a XRTDevice,
    kernel: &'a XRTKernel,
}

impl ManagedRun<'_> {
    pub fn new<'a>(device: &'a ManagedDevice, kernel: &'a XRTKernel) -> Result<ManagedRun<'a>> {
        Ok(ManagedRun {
            run: kernel.run()?,
            buffers: HashMap::new(),
            device: &device.device,
            kernel,
        })
    }

    pub fn start(self) -> Result<Self> {
        self.run.start()?;
        Ok(self)
    }

    pub fn wait_for(self, timeout_ms: u32) -> Result<Self> {
        self.run.wait_for(timeout_ms)?;
        Ok(self)
    }

    pub fn set_scalar_input<T: HardwareDatatype>(self, index: i32, value: T) -> Result<Self> {
        self.run.set_scalar_argument(index, value)?;
        Ok(self)
    }

    pub fn set_buffer_input<T: HardwareDatatype>(
        mut self,
        index: i32,
        values: &[T],
    ) -> Result<Self> {
        let buffer = self
            .run
            .write_buffer_argument(index, values, self.device, self.kernel)?;
        self.run.set_buffer_argument(index, &buffer)?;
        self.buffers.insert(index, buffer);
        Ok(self)
    }

    pub fn prepare_output_buffer<T: HardwareDatatype>(
        mut self,
        index: i32,
        size: usize,
    ) -> Result<Self> {
        let buffer = self
            .run
            .create_read_buffer::<T>(index, size, &self.device, self.kernel)?;
        self.run.set_buffer_argument(index, &buffer)?;
        self.buffers.insert(index, buffer);

        return Ok(self);
    }

    pub fn read_output<T: HardwareDatatype>(
        mut self,
        index: i32,
        values: &mut [T],
    ) -> Result<Self> {
        if let Some(buffer) = self.buffers.get(&index) {
            self.run
                .read_buffer_argument(buffer, values.len(), values)?;
            return Ok(self);
        } else {
            return Err(Error::BONotCreatedYet);
        }
    }
}

pub struct ManagedDevice {
    device: XRTDevice,
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
