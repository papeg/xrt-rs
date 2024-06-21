use std::collections::HashMap;

use crate::managed::device::ManagedDevice;
use crate::native::buffer::XRTBuffer;
use crate::native::device::XRTDevice;
use crate::native::kernel::XRTKernel;
use crate::native::run::XRTRun;
use crate::HardwareDatatype;
use crate::{Error, Result};
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
