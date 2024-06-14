use crate::components::common::{is_null, ERTCommandState, XRTError};
use crate::components::kernel::XRTKernel;
use crate::ffi::*;

pub struct XRTRun {
    handle: Option<xrtRunHandle>,
}

impl XRTRun {
    pub fn new(kernel: &XRTKernel) -> Result<Self, XRTError> {
        let handle = unsafe {
            xrtRunOpen(
                kernel
                    .get_handle()
                    .ok_or(XRTError::KernelNotLoadedYetError)?,
            )
        };
        if is_null(handle) {
            return Err(XRTError::RunCreationError);
        }
        Ok(XRTRun {
            handle: Some(handle),
        })
    }

    pub fn set_argument<T>(&mut self, argument_number: u32, value: T) -> Result<(), XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::RunNotCreatedYetError);
        }
        let result = unsafe { xrtRunSetArg(self.handle.unwrap(), argument_number as i32, value) };
        if result != 0 {
            return Err(XRTError::SetRunArgError);
        }
        Ok(())
    }

    /// Get the current ERTCommandState of the run. Returns an error if called before this run is properly initialized
    pub fn get_state(&self) -> Result<ERTCommandState, XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::RunNotCreatedYetError);
        }
        Ok(ERTCommandState::from(unsafe {
            xrtRunState(self.handle.unwrap())
        }))
    }

    /// Start a run. Optionally wait for the run to finish within the given timeout. If not waiting,
    /// the timeout is ignored. Returns the Command State after starting / finishing the run
    pub fn start(&self, wait: bool, wait_timeout_ms: u32) -> Result<ERTCommandState, XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::RunNotCreatedYetError);
        }
        let run_res = unsafe { xrtRunStart(self.handle.unwrap()) };
        if run_res != 0 {
            return Ok(self.get_state()?); //? Return Ok(state) or an Err? Probably Ok(state) because the state might contain the reason for the failed run start
        }
        if !wait {
            return Ok(self.get_state()?);
        }
        Ok(ERTCommandState::from(unsafe {
            xrtRunWaitFor(self.handle.unwrap(), wait_timeout_ms)
        }))
    }
}

impl Drop for XRTRun {
    fn drop(&mut self) {
        if self.handle.is_some() {
            unsafe {
                xrtRunClose(self.handle.unwrap());
            }
            self.handle = None
        }
    }
}
