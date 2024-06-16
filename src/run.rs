use crate::buffer::{SyncDirection, XRTBuffer};
use crate::device::XRTDevice;
use crate::ffi::*;
use crate::kernel::XRTKernel;
use crate::utils::is_null;
use crate::{Error, Result};
use std::collections::HashMap;

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
    internal_buffers: HashMap<i32, XRTBuffer>,
}

impl XRTRun {
    pub fn new(kernel: &XRTKernel) -> Result<Self> {
        if let Some(kernel_handle) = kernel.handle {
            let run_handle = unsafe { xrtRunOpen(kernel_handle) };
            if is_null(run_handle) {
                return Err(Error::RunCreationError);
            }
            Ok(XRTRun {
                handle: Some(run_handle),
                internal_buffers: HashMap::new(),
            })
        } else {
            return Err(Error::KernelNotLoadedYetError);
        }
    }

    pub fn set_scalar_argument<T>(&mut self, index: i32, value: T) -> Result<()> {
        if let Some(handle) = self.handle {
            let result = unsafe { xrtRunSetArg(handle, index, value) };
            if result != 0 {
                return Err(Error::SetRunArgError);
            }
            Ok(())
        } else {
            return Err(Error::RunNotCreatedYetError);
        }
    }

    pub fn set_buffer_argument(&mut self, index: i32, buffer: &XRTBuffer) -> Result<()> {
        if let Some(run_handle) = self.handle {
            if let Some(buffer_handle) = buffer.handle {
                let result = unsafe { xrtRunSetArg(run_handle, index, buffer_handle) };
                if result != 0 {
                    return Err(Error::SetRunArgError);
                }
                Ok(())
            } else {
                return Err(Error::BONotCreatedYet);
            }
        } else {
            return Err(Error::RunNotCreatedYetError);
        }
    }

    pub fn write_buffer_argument<T>(
        &mut self,
        index: i32,
        values: &[T],
        device: &XRTDevice,
        kernel: &XRTKernel,
    ) -> Result<()> {
        let buffer = XRTBuffer::new(
            &device,
            values.len() * std::mem::size_of::<T>(),
            XCL_BO_FLAGS_NONE,
            kernel.get_memory_group_for_argument(index)?,
        )?;

        buffer.write(values, 0)?;
        buffer.sync(SyncDirection::HostToDevice, None, 0)?;

        self.internal_buffers.insert(index, buffer);

        Ok(())
    }

    pub fn create_read_buffer<T>(
        &mut self,
        index: i32,
        size: usize,
        device: &XRTDevice,
        kernel: &XRTKernel,
    ) -> Result<()> {
        let buffer = XRTBuffer::new(
            &device,
            size * std::mem::size_of::<T>(),
            XCL_BO_FLAGS_NONE,
            kernel.get_memory_group_for_argument(index)?,
        )?;

        self.internal_buffers.insert(index, buffer);
        Ok(())
    }

    pub fn read_buffer_argument<T>(&mut self, index: i32, size: usize) -> Result<Vec<T>> {
        if let Some(buffer) = self.internal_buffers.get(&index) {
            let mut output: Vec<T> = Vec::with_capacity(size);
            buffer.sync(SyncDirection::DeviceToHost, None, 0)?;
            buffer.read(&mut output, 0)?;
            Ok(output)
        } else {
            Err(Error::BONotCreatedYet)
        }
    }

    /// Get the current ERTCommandState of the run. Returns an error if called before this run is properly initialized
    pub fn get_state(&self) -> Result<ERTCommandState> {
        if let Some(handle) = self.handle {
            Ok(ERTCommandState::from(unsafe { xrtRunState(handle) }))
        } else {
            return Err(Error::RunNotCreatedYetError);
        }
    }

    /// Start a run. Optionally wait for the run to finish within the given timeout. If not waiting,
    /// the timeout is ignored. Returns the Command State after starting / finishing the run
    pub fn start(&self) -> Result<ERTCommandState> {
        if let Some(handle) = self.handle {
            let run_res = unsafe { xrtRunStart(handle) };
            if run_res != 0 {
                return Ok(self.get_state()?); //? Return Ok(state) or an Err? Probably Ok(state) because the state might contain the reason for the failed run start
            }
            return Ok(self.get_state()?);
        } else {
            return Err(Error::RunNotCreatedYetError);
        }
    }

    pub fn wait_for(&self, timeout_ms: u32) -> Result<ERTCommandState> {
        if let Some(handle) = self.handle {
            Ok(unsafe { ERTCommandState::from(xrtRunWaitFor(handle, timeout_ms)) })
        } else {
            Err(Error::RunNotCreatedYetError)
        }
    }

    pub fn wait(&self) -> Result<ERTCommandState> {
        if let Some(handle) = self.handle {
            Ok(unsafe { ERTCommandState::from(xrtRunWait(handle)) })
        } else {
            Err(Error::RunNotCreatedYetError)
        }
    }
}

impl Drop for XRTRun {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            unsafe {
                xrtRunClose(handle);
            }
        }
    }
}
