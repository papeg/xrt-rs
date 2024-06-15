use crate::buffer::XRTBuffer;
use crate::ffi::*;
use crate::kernel::XRTKernel;
use crate::utils::is_null;
use crate::{Error, Result};

/// Every state value that a run can have. These are ususally parsed from the u32 returned from the C-interface
#[derive(Debug, PartialEq)]
pub enum ERTCommandState {
    Completed,
    InvalidState(u32),
    Abort,
    Error,
    Queued,
    Running,
    NoResponse,
    Submitted,
    New,
    Max,
    Timeout,
    SKError,
    SKCrashed,
}

impl From<u32> for ERTCommandState {
    fn from(value: u32) -> Self {
        //? Replace by macro?
        match value {
            ert_cmd_state_ERT_CMD_STATE_COMPLETED => ERTCommandState::Completed,
            ert_cmd_state_ERT_CMD_STATE_ABORT => ERTCommandState::Abort,
            ert_cmd_state_ERT_CMD_STATE_ERROR => ERTCommandState::Error,
            ert_cmd_state_ERT_CMD_STATE_QUEUED => ERTCommandState::Queued,
            ert_cmd_state_ERT_CMD_STATE_RUNNING => ERTCommandState::Running,
            ert_cmd_state_ERT_CMD_STATE_NORESPONSE => ERTCommandState::NoResponse,
            ert_cmd_state_ERT_CMD_STATE_SUBMITTED => ERTCommandState::Submitted,
            ert_cmd_state_ERT_CMD_STATE_NEW => ERTCommandState::New,
            ert_cmd_state_ERT_CMD_STATE_MAX => ERTCommandState::Max,
            ert_cmd_state_ERT_CMD_STATE_TIMEOUT => ERTCommandState::Timeout,
            ert_cmd_state_ERT_CMD_STATE_SKCRASHED => ERTCommandState::SKCrashed,
            ert_cmd_state_ERT_CMD_STATE_SKERROR => ERTCommandState::SKError,
            _ => ERTCommandState::InvalidState(value),
        }
    }
}

pub struct XRTRun {
    pub(crate) handle: Option<xrtRunHandle>,
}

impl XRTRun {
    pub fn new(kernel: &XRTKernel) -> Result<Self> {
        let handle =
            unsafe { xrtRunOpen(kernel.get_handle().ok_or(Error::KernelNotLoadedYetError)?) };
        if is_null(handle) {
            return Err(Error::RunCreationError);
        }
        Ok(XRTRun {
            handle: Some(handle),
        })
    }

    pub fn set_scalar_argument<T>(&mut self, index: i32, value: T) -> Result<()> {
        if self.handle.is_none() {
            return Err(Error::RunNotCreatedYetError);
        }
        let result = unsafe { xrtRunSetArg(self.handle.unwrap(), index, value) };
        if result != 0 {
            return Err(Error::SetRunArgError);
        }
        Ok(())
    }

    pub fn set_buffer_argument(&mut self, index: i32, buffer: &XRTBuffer) -> Result<()> {
        if self.handle.is_none() {
            return Err(Error::RunNotCreatedYetError);
        }
        if let Some(handle) = buffer.handle {
            let result = unsafe { xrtRunSetArg(self.handle.unwrap(), index, handle) };
            if result != 0 {
                return Err(Error::SetRunArgError);
            }
            Ok(())
        } else {
            return Err(Error::BONotCreatedYet);
        }
    }

    /// Get the current ERTCommandState of the run. Returns an error if called before this run is properly initialized
    pub fn get_state(&self) -> Result<ERTCommandState> {
        if self.handle.is_none() {
            return Err(Error::RunNotCreatedYetError);
        }
        Ok(ERTCommandState::from(unsafe {
            xrtRunState(self.handle.unwrap())
        }))
    }

    /// Start a run. Optionally wait for the run to finish within the given timeout. If not waiting,
    /// the timeout is ignored. Returns the Command State after starting / finishing the run
    pub fn start(&self, wait: bool, wait_timeout_ms: u32) -> Result<ERTCommandState> {
        if self.handle.is_none() {
            return Err(Error::RunNotCreatedYetError);
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
