use std::collections::{HashMap, VecDeque};

use crate::buffer::{SyncDirection, XRTBuffer};
use crate::device::XRTDevice;
use crate::ffi::*;
use crate::kernel::XRTKernel;
use crate::run::{ERTCommandState, XRTRun};
use crate::utils::is_null;
use crate::{Error, Result};


pub enum ArgumentType {
    Scalar,
    Buffer(XRTBuffer)
}

pub enum Argument<T> {
    Single(T),
    Vec(Vec<T>),
}

/// Idea: Make life easier for the user. The usage is kind of imagined like this:
/// ```
/// let values = vec![1,2,3,4];
/// let dm = DeviceManager::new()
///     .with_xclbin("scaler.xclbin")?
///     .with_kernel("vscale")?
///     .run("vscale", arguments!(4, values))?
///     .wait_on_latest_run()?;
/// let result_state = dm.get_latest_run_state()?;
/// dm.remove_latest_run()?
///     .run(...); 
/// ```
/// 
/// Some features for that are still missing, for example automatically retrieving what types the arguments of a kernel are
pub struct DeviceManager {
    device: XRTDevice,
    kernels: HashMap<String, (XRTKernel, Vec<ArgumentType>)>,
    
    /// A vec of open runs. Gets added to when a run is started
    open_runs: VecDeque<XRTRun>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager { 
            device: XRTDevice::new(), 
            kernels: HashMap::new(), 
            open_runs: VecDeque::new()
        }
    }

    pub fn with_xclbin(mut self, xclbin_path: &str) -> Result<Self> {
        self.device.load_xclbin(xclbin_path)?;
        Ok(self)
    }

    // TODO: Urgent: Extract info about arguments for kernels from xclbin!
    pub fn with_kernel(mut self, kernel_name: &str, arglist: Vec<ArgumentType>) -> Result<Self> {
        let kernel = XRTKernel::new(kernel_name, &self.device)?;        
        self.kernels.insert(kernel_name.to_string(), (kernel, arglist));
        Ok(self)
    }

    /// The idea here is that you can pass in the arguments, and the function takes care of whether it has to be written into a buffer first
    /// or can be passed directly. This enables you to pass in data easily:
    /// ```
    /// let my_values = vec![1,2,3,4];
    /// let my_scale = 4;
    /// 
    /// dm.run("vscale", &[Argument::Single(my_scale), Argument::Vec(my_values)]);
    /// ```
    /// 
    /// **TODO**: Make a macro to avoid having to construct an enum everytime: dm.run("vscale", my_scale, my_values);
    pub fn run<T: Clone>(mut self, kernel_name: &str, arguments: &[Argument<T>]) -> Result<Self> {
        let (kernel, arg_types) = self.kernels.get(kernel_name).ok_or(Error::NoSuchKernelError)?;
        if arguments.len() != arg_types.len() {
            return Err(Error::ArgumentNumberMismatchError);
        }
        let mut run = XRTRun::new(kernel)?;

        for i in 0..arguments.len() {
            if let ArgumentType::Buffer(b) = &arg_types[i] {
                let data = match &arguments[i] {
                    Argument::Single(d) => vec![d.clone()],
                    Argument::Vec(d) => d.clone()
                };

                b.write(&data, 0)?;
                b.sync(SyncDirection::DeviceToHost, Some(data.len()), 0)?;
            } else {
                let data = match &arguments[i] {
                    Argument::Single(d) => d,
                    Argument::Vec(_) => return Err(Error::PassVecToScalarArgumentError)
                };
                run.set_scalar_argument(i as i32, data)?;
            }
        }
        
        self.open_runs.push_back(run);
        Ok(self)
    }

    pub fn wait_on_latest_run(mut self) -> Result<Self> {
        let result = self.open_runs.back().ok_or(Error::NoOpenRunsError)?.wait();
        match result {
            Ok(_) => Ok(self),
            Err(e) => Err(e)
        }
    }

    pub fn get_latest_run_state(mut self) -> Result<ERTCommandState> {
        self.open_runs.back().ok_or(Error::NoOpenRunsError)?.get_state()
    }

    pub fn remove_latest_run(mut self) -> Result<Self> {
        self.open_runs.pop_front();
        Ok(self)
    }

}