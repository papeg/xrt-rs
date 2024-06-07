include!("bindings_c.rs");
use std::{collections::HashMap, hash::Hash, os::raw::c_void, rc::*};

/// Helper func to return if a given handle is null
fn is_null(handle: *mut c_void) -> bool {
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
type ArgumentIndex = u32;

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

pub struct XRTKernel {
    kernel_handle: xrtKernelHandle,
    
    /// A mapping to describe how a kernel has to be called. For every argument index specifies whether a buffer handle is used (which is prepared at
    /// construction time of the XRTKernel) or whether it is left blank and requires input from XRT when calling the kernel. This is also
    /// the reason why this struct doenst need to save the buffer handles explicitly
    argument_mapping: HashMap<ArgumentIndex, ArgumentType>
}

impl XRTKernel {
    /// Construct a new XRTKernel. This guards against acidentally not having initialized all required buffers
    pub fn new(kernel_handle: xrtKernelHandle, argument_mapping: HashMap<ArgumentIndex, ArgumentType>) -> Result<Self, XRTError> {
        if !XRTKernel::is_ready(&argument_mapping) {
            return Err(XRTError::UnrealizedBufferError);
        }
        Ok(
            XRTKernel {
                kernel_handle: kernel_handle,
                argument_mapping: argument_mapping
            }
        )
    }

    /// Tells whether the XRTKernel is ready for execution. If not, its argument mapping has to be edited
    pub fn is_ready(argument_mapping: &HashMap<ArgumentIndex, ArgumentType>) -> bool {
        !argument_mapping.values().any(|x| matches!(x, ArgumentType::NotRealizedBuffer(_)))
    }

    // Create a run. Doing this does not execute any action. It just prepares the arguments
    pub fn create_run(&self, argument_data: HashMap<ArgumentIndex, Argument>) -> Result<XRTRun, XRTError> {
        if argument_data.len() != self.argument_mapping.len() {
            return Err(XRTError::NonMatchingArgumentLists);
        }
        let run_handle = unsafe { xrtRunOpen(self.kernel_handle) };
        if is_null(run_handle) {
            return Err(XRTError::RunCreationError);
        }
        Ok(
            XRTRun { run_handle: run_handle, argument_mapping: &self.argument_mapping, argument_data: argument_data }
        )
    }
}


/// Struct to manage runs. Creating a run does not start it. The current state of a given run can be checked on.
/// **Key idea** is to give the data to synchronize directly to a run instead of the buffers. When the run is started it can automatically
/// transfer the data. The user can advise it to do so at any point as well
pub struct XRTRun<'a> {
    run_handle: xrtRunHandle,
    argument_mapping: &'a HashMap<ArgumentIndex, ArgumentType>,   // What kind of arguments and where to sync to
    argument_data: HashMap<ArgumentIndex, Argument>               // The content of the arguments itself
}

/// This impl does not contain a constructor because a valid run can and should only be constructed from a device!
impl<'a> XRTRun<'a> {
    /// Return the current state of the run
    pub fn get_state(&self) -> ERTCommandState {
        let state: u32 = unsafe { xrtRunState(self.run_handle) };
        ERTCommandState::from(state)
    }

    /// This sets the direct arguments of the run and fills the buffers with the data and syncs them. If this is done without
    /// executing the kernel, another XRTRun might do the same and overwrite the data. 
    /// 
    /// __This is manually called by `start_run()`. So in many cases you will want to use that convenience method and
    /// only use this one in case you need to have more precise control.__
    pub fn load_arguments(&self) -> Result<(), XRTError> {
        if self.argument_data.len() != self.argument_mapping.len() {
            return Err(XRTError::NonMatchingArgumentLists);
        }

        for (index, value) in self.argument_data {
            match value {
                Argument::Direct(data) => { 
                    let result = unsafe { xrtRunSetArg(self.run_handle, index as i32) };
                    if result != 0 {
                        return Err(XRTError::RunArgumentSetError(index, data));
                    }
                },

                Argument::BufferContent(data) =>  {
                    // Get the buffer handle
                    let buffer_handle_result = self.argument_mapping.get(&index).ok_or(XRTError::InvalidArgumentIndex)?;
                    let buffer_handle = match buffer_handle_result {
                        ArgumentType::Buffer(bhdl) => *bhdl,
                        _ => return Err(XRTError::ExpectedBufferArgumentType)
                    };

                    // Write data to fpga
                    let write_result = unsafe { xrtBOWrite(buffer_handle, data.as_ptr() as *mut std::os::raw::c_void, data.len(), 0) };
                    if write_result != 0 {
                        return Err(XRTError::FailedBOWrite);
                    }

                    // Sync to FPGA
                    let sync_result = unsafe { xrtBOSync(buffer_handle, xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE, data.len(), 0) };
                    if sync_result != 0 {
                        return Err(XRTError::FailedBOSyncToDevice);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn start_run() {}
}

#[allow(dead_code)]
pub struct XRTDevice<'a> {
    device_handle: Option<xrtDeviceHandle>,
    xclbin_handle: Option<xrtXclbinHandle>,
    xclbin_uuid: Option<xuid_t>,
    kernel_handles: HashMap<String, XRTKernel>,
    run_handles: HashMap<String, Vec<Rc<XRTRun<'a>>>>,
}

// TODO: Make generic for all types that implement Num Trait
impl TryFrom<u32> for XRTDevice<'_> {
    type Error = XRTError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let dh = XRTDevice {
            device_handle: Some(unsafe { xrtDeviceOpen(value) }),
            xclbin_handle: None,
            xclbin_uuid: None,
            kernel_handles: HashMap::new(),
            run_handles: HashMap::new(),
        };
        if is_null(dh.get_handle()?) {
            return Err(XRTError::InvalidDeviceIDError);
        }
        Ok(dh)
    }
}

#[allow(dead_code)]
impl<'a> XRTDevice<'a> {
    // -- Builder Interface --
    pub fn from_index(index: u32) -> Result<XRTDevice<'a>, XRTError> {
        XRTDevice::try_from(index)
    }

    pub fn with_xclbin(mut self, filepath: &str) -> Result<XRTDevice, XRTError> {
        self.load_xclbin(filepath)?;
        Ok(self)
    }

    pub fn with_kernel(mut self, name: &str) -> Result<XRTDevice, XRTError> {
        self.load_kernel(name)?;
        Ok(self)
    }

    // -- Methods --
    /// Opens a device with a given index. If the device handle was set before, the loaded Xclbin, UUID and kernels get deleted. Returns an error if the opening failed.
    pub fn open_device(&mut self, index: u32) -> Result<(), XRTError> {
        if self.device_handle.is_some() {
            self.xclbin_handle = None;
            self.xclbin_uuid = None;
            self.kernel_handles.clear();
        }
        self.device_handle = Some(unsafe { xrtDeviceOpen(index) });
        if is_null(self.device_handle.unwrap()) {
            return Err(XRTError::InvalidDeviceIDError);
        } else {
            return Ok(());
        }
    }

    /// Checks whether the XRTRSDevice is ready to execute kernels. This requires a loaded xclbin, a correctly initialized device and
    /// a set UUID
    pub fn is_ready(&self) -> bool {
        self.xclbin_handle.is_some() && self.xclbin_uuid.is_some() && self.device_handle.is_some()
    }

    pub fn get_handle(&self) -> Result<xrtDeviceHandle, XRTError> {
        // Possible because we never even construct a faulty device handle
        match self.device_handle {
            None => Err(XRTError::NoDeviceLoadedError),
            Some(dh) => Ok(dh),
        }
    }

    /// Sets the UUID of the Xclbin. This requires a valid device handle and a loaded xclbin
    fn set_uuid(&mut self) -> Result<(), XRTError> {
        match self.xclbin_handle {
            None => Err(XRTError::XclbinNotLoadedError),
            Some(xclbinhandle) => {
                let mut raw_uuid: xuid_t = [0; 16];
                let retval = unsafe { xrtXclbinGetUUID(xclbinhandle, raw_uuid.as_mut_ptr()) };
                if retval != 0 {
                    return Err(XRTError::UUIDRetrievalError);
                }
                self.xclbin_uuid = Some(raw_uuid);
                return Ok(());
            }
        }
    }

    /// Load the xclbin by filename and additionally set the UUID member of the XRTRSDevice.
    pub fn load_xclbin(&mut self, filepath: &str) -> Result<(), XRTError> {
        // 1. Alloc the xclbin filename
        // 2. Load the xclbin onto the device
        // 3. Set UUID for the loaded XCLBIN
        let cstring_path = std::ffi::CString::new(filepath)
            .expect("Failed to create cstring from given filepath!");

        let handle: xrtXclbinHandle = unsafe { xrtXclbinAllocFilename(cstring_path.as_ptr()) };
        if is_null(handle) {
            return Err(XRTError::XclbinFilenameAllocError);
        }

        if unsafe { xrtDeviceLoadXclbinHandle(self.device_handle.unwrap(), handle) } != 0 {
            return Err(XRTError::XclbinLoadingError);
        }

        // Only now set the structs handle, in case the loading failed
        self.xclbin_handle = Some(handle);

        // Set the uuid of the xclbin
        self.set_uuid()?;
        Ok(())
    }

    /// Load a kernel by name. This name is then used to store it in a XRTRSDevice internal hashmap
    fn load_kernel(&mut self, name: &str, initial_argument_mapping: HashMap<ArgumentIndex, ArgumentType>) -> Result<(), XRTError> {
        // If XCLBIN and UUID are set, load and store a handle to the specified kernel by it's name
        let raw_kernel_name =
            std::ffi::CString::new(name).expect("Error on creation of kernel name string!");
        if self.xclbin_handle == None {
            return Err(XRTError::XclbinNotLoadedError);
        }

        let kernel_handle: xrtKernelHandle;
        let mut uuid = self.xclbin_uuid.ok_or(XRTError::GeneralError(
            "Cannot set kernel handler for a device without an XCLBIN handler".to_string(),
        ))?;

        // Open kernel
        kernel_handle = unsafe {
            xrtPLKernelOpen(
                self.device_handle.unwrap(),
                uuid.as_mut_ptr(),
                raw_kernel_name.as_ptr(),
            )
        };
        if is_null(kernel_handle) {
            return Err(XRTError::KernelCreationError);
        }


        // Creating necessary buffer objects
        let mut argument_mapping: HashMap<ArgumentIndex, ArgumentType> = HashMap::new();
        for (k,v) in initial_argument_mapping {
            if let ArgumentType::NotRealizedBuffer(required_size) = v {
                let group_id_handle = unsafe { 
                    xrtKernelArgGroupId(kernel_handle, k as i32) 
                }; 
                
                if group_id_handle < 0 {
                    return Err(XRTError::InvalidGroupIDError);
                }

                let bo_handle = unsafe { 
                    xrtBOAlloc(
                        self.device_handle.unwrap(), 
                        required_size as usize, 
                        XCL_BO_FLAGS_NONE as std::os::raw::c_ulong, 
                        group_id_handle as std::os::raw::c_uint
                    )
                };

                if is_null(bo_handle) {
                    return Err(XRTError::FailedBOAllocError);
                }

                argument_mapping.insert(k, ArgumentType::Buffer(bo_handle));
            } else {
                argument_mapping.insert(k, v);
            }
        }


        // Construct new kernel object
        let xrtkernel = XRTKernel::new(kernel_handle, argument_mapping)?;
        self.kernel_handles.insert(name.to_string(), xrtkernel);
        Ok(())
    }


}

impl Drop for XRTDevice<'_> {
    fn drop(&mut self) {
        // TODO: Deallocate any buffers
        // Close runs
        for kernel_name in self.run_handles.keys() {
            for run_handle in &self.run_handles[kernel_name] {
                unsafe { xrtRunClose(run_handle.run_handle) };
            }
        }

        // Close kernels
        for kernel in self.kernel_handles.values() {
            unsafe { xrtKernelClose(*kernel) };
        }

        // Make sure to not try to close a non-open device
        if self.device_handle.is_some() {
            unsafe { xrtDeviceClose(self.device_handle.unwrap()) };
        }
    }
}

// Tests
#[test]
fn emu_open_device_test() -> Result<(), XRTError> {
    let device = XRTDevice::from_index(0)?;
    assert!(device.device_handle.is_some());
    Ok(())
}

#[test]
fn emu_open_device_load_xclbin_test() -> Result<(), XRTError> {
    use crate::get_xclbin_path;

    let mut device = XRTDevice::from_index(0)?;
    assert!(device.device_handle.is_some());
    let xclbin_path = get_xclbin_path("add");
    device.load_xclbin(&xclbin_path)?;
    assert!(device.xclbin_handle.is_some());
    assert!(device.xclbin_uuid.is_some());

    Ok(())
}

#[test]
fn emu_open_device_load_xclbin_builder_test() -> Result<(), XRTError> {
    use crate::get_xclbin_path;

    let xclbin_path = get_xclbin_path("add");
    let device = XRTDevice::from_index(0)?
        .with_xclbin(&xclbin_path)?
        .with_kernel("add")?;

    assert!(device.device_handle.is_some());
    assert!(device.xclbin_handle.is_some());
    assert!(device.xclbin_uuid.is_some());

    Ok(())
}
