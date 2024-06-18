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
pub struct ActiveRun {
    run: XRTRun,
    buffers: HashMap<i32, XRTBuffer>,
}

impl From<XRTRun> for ActiveRun {
    fn from(run: XRTRun) -> ActiveRun {
        ActiveRun {
            run,
            buffers: HashMap::new(),
        }
    }
}

pub struct DeviceManager {
    device: XRTDevice,
    kernels: HashMap<String, XRTKernel>,
    runs: HashMap<String, Vec<ActiveRun>>,
    current_kernel: Option<String>,
    current_run: Option<usize>,
}

impl From<XRTDevice> for DeviceManager {
    fn from(device: XRTDevice) -> DeviceManager {
        DeviceManager {
            device: device,
            kernels: HashMap::new(),
            runs: HashMap::new(),
            current_kernel: None,
            current_run: None,
        }
    }
}

impl DeviceManager {
    pub fn with_xclbin(mut self, xclbin_path: &str) -> Result<Self> {
        self.device.load_xclbin(xclbin_path)?;
        Ok(self)
    }

    pub fn with_kernel(mut self, kernel_name: &str) -> Result<Self> {
        let kernel = XRTKernel::new(kernel_name, &self.device)?;
        self.kernels.insert(kernel_name.to_string(), kernel);
        Ok(self)
    }

    pub fn prepare_run(mut self, kernel_name: &str) -> Result<Self> {
        if let Some(kernel) = self.kernels.get(kernel_name) {
            self.current_kernel = Some(kernel_name.into());
            if let Some(runs) = self.runs.get_mut(kernel_name) {
                runs.push(ActiveRun::from(kernel.run()?));
                self.current_run = Some(runs.len() - 1);
            } else {
                let mut runs: Vec<ActiveRun> = Vec::new();
                runs.push(ActiveRun::from(kernel.run()?));
                self.runs.insert(kernel_name.into(), runs);
                self.current_run = Some(0);
            }
            Ok(self)
        } else {
            return Err(Error::KernelNotLoadedYetError);
        }
    }

    pub fn set_input<T: HardwareDatatype>(
        mut self,
        index: i32,
        value: ArgumentType<T>,
    ) -> Result<Self> {
        if let Some(kernel_name) = &self.current_kernel {
            if let Some(kernel) = self.kernels.get(kernel_name.into()) {
                if let Some(current_run) = self.current_run {
                    if let Some(runs) = self.runs.get_mut(kernel_name.into()) {
                        let run = &mut runs[current_run];
                        match value {
                            ArgumentType::Scalar(value) => {
                                run.run.set_scalar_argument(index, value)?;
                            }
                            ArgumentType::Buffer(values) => {
                                run.buffers.insert(
                                    index,
                                    run.run.write_buffer_argument(
                                        index,
                                        values,
                                        &self.device,
                                        kernel,
                                    )?,
                                );
                            }
                        }
                        return Ok(self);
                    }
                }
                return Err(Error::RunNotCreatedYetError);
            }
        }
        Err(Error::KernelNotLoadedYetError)
    }

    pub fn set_scalar_input<T: HardwareDatatype>(self, index: i32, value: T) -> Result<Self> {
        self.set_input(index, ArgumentType::Scalar(value))
    }

    pub fn set_buffer_input<T: HardwareDatatype>(self, index: i32, values: &[T]) -> Result<Self> {
        self.set_input(index, ArgumentType::Buffer(values))
    }

    pub fn prepare_output_buffer<T: HardwareDatatype>(
        mut self,
        index: i32,
        size: usize,
    ) -> Result<Self> {
        if let Some(kernel_name) = &self.current_kernel {
            if let Some(kernel) = self.kernels.get(kernel_name.into()) {
                if let Some(current_run) = self.current_run {
                    if let Some(runs) = self.runs.get_mut(kernel_name.into()) {
                        let run = &mut runs[current_run];
                        run.buffers.insert(
                            index,
                            run.run
                                .create_read_buffer::<T>(index, size, &self.device, kernel)?,
                        );

                        return Ok(self);
                    }
                }
                return Err(Error::RunNotCreatedYetError);
            }
        }
        Err(Error::KernelNotLoadedYetError)
    }

    pub fn start_all(self) -> Result<Self> {
        for runs in self.runs.values() {
            for run in runs {
                run.run.start()?;
            }
        }
        Ok(self)
    }

    pub fn wait_for_all(self, timeout_ms: u32) -> Result<Self> {
        for runs in self.runs.values() {
            for run in runs {
                run.run.wait_for(timeout_ms)?;
            }
        }
        Ok(self)
    }

    pub fn read_output<T: HardwareDatatype>(
        mut self,
        index: i32,
        values: &mut [T],
    ) -> Result<Self> {
        if let Some(kernel_name) = &self.current_kernel {
            if let Some(current_run) = self.current_run {
                if let Some(runs) = self.runs.get_mut(kernel_name.into()) {
                    let run = &mut runs[current_run];
                    if let Some(buffer) = run.buffers.get(&index) {
                        run.run.read_buffer_argument(buffer, values.len(), values)?;
                        return Ok(self);
                    } else {
                        return Err(Error::BONotCreatedYet);
                    }
                }
            }
            return Err(Error::RunNotCreatedYetError);
        }
        Err(Error::KernelNotLoadedYetError)
    }
}
