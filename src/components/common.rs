include!("../bindings_c.rs");
use std::{collections::HashMap, hash::Hash, os::raw::c_void, rc::*};


/// Helper func to return if a given handle is null
pub fn is_null(handle: *mut c_void) -> bool {
    handle == (std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void)
}

#[derive(Debug)]
pub enum XRTError {
    GeneralError(String),
    XclbinNotLoadedError, // For when an XCLBIN is required but not present
    NoDeviceLoadedError,
    InvalidDeviceIDError,
    UUIDRetrievalError,
    XclbinFilenameAllocError,
    XclbinLoadingError, // For when the loading of the XCLBIN itself fails
    KernelCreationError,
    MissingKernelError,
    RunCreationError,
    RunArgumentSetError(ArgumentIndex, i32), // Pass argument index and value. Required here, because one call might set different args, so we have to hold that information
    UnrealizedBufferError, // Tried creating a kernel without constructing all required BOs first
    InvalidGroupIDError,
    FailedBOAllocError,
    NonMatchingArgumentLists, // For when the Argument mappings and contents of an XRTRun dont agree in length
    InvalidArgumentIndex,
    ExpectedBufferArgumentType, // For when an argument is passed to fill a buffer, but the argument mapping requires a direct pass
    FailedBOWrite,
    FailedBOSyncToDevice,
}

/// Every state value that a run can have. These are ususally parsed from the u32 returned from the C-interface
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

/// Used to store the mapping of arguments per kernel. It defines an argument to either be taken as a buffer address/handle (returned from xrtKernelArgGroupId)
/// or to be passed when constructing a run
pub enum ArgumentType {
    Buffer(xrtBufferHandle),
    Passed,
    NotRealizedBuffer(u32) // Represents a not yet initialized buffer of the given u32 size. A valid mapping of a kernel does not contain this variant
}

/// This enum is used to store how the argument is supposed to be used when creating a run. The difference to `ArgumentType` is, that
/// this one specifies the arguments for a run, but `ArgumentType` specifies for which arguments a buffer to create and what their handle is
pub enum Argument {
    Direct(i32),
    BufferContent(Vec<i8>)
}