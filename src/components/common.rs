include!("../bindings_c.rs");
use std::{os::raw::c_void, rc::*};



/// Helper func to return if a given handle is null
pub fn is_null(handle: *mut c_void) -> bool {
    handle == (std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void)
}

#[derive(Debug)]
pub enum XRTError {
    GeneralError(String),
    DeviceOpenError,
    UnopenedDeviceError,
    CStringCreationError,
    XclbinFileAllocError,
    XclbinLoadError,
    XclbinUUIDRetrievalError,
    DeviceNotReadyError,
    KernelCreationError,
    KernelNotLoadedYetError,
    KernelArgRtrvError,
    BOCreationError,
}

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

/// Represents an index of where to put arguments
pub type ArgumentIndex = u32;

pub enum IOMode {
    Input,
    Output
}