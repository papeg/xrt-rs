#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
include!("bindings_c.rs");
// tests for cpp bindings are failing
//include!("bindings_cpp.rs");

use std::collections::HashMap;

#[test]
fn run_kernel_raw() {
    std::env::set_var("XCL_EMULATION_MODE", "sw_emu");

    let device_handle: xrtDeviceHandle = unsafe { xrtDeviceOpen(0) };

    assert_ne!(
        device_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let xclbin_path =
        std::ffi::CString::new("add_sw_emu.xclbin").expect("creating CString for xclbin_path");

    let xclbin_handle = unsafe { xrtXclbinAllocFilename(xclbin_path.as_ptr() as *const i8) };

    assert_ne!(
        xclbin_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    assert_eq!(
        unsafe { xrtDeviceLoadXclbinHandle(device_handle, xclbin_handle) },
        0,
    );

    let mut xclbin_uuid: xuid_t = [0; 16];

    assert_eq!(
        unsafe { xrtXclbinGetUUID(xclbin_handle, xclbin_uuid.as_mut_ptr()) },
        0,
    );

    let kernel_name = std::ffi::CString::new("add").expect("creating CString for kernel name");

    let add_kernel_handle: xrtKernelHandle = unsafe {
        xrtPLKernelOpen(
            device_handle,
            xclbin_uuid.as_mut_ptr(),
            kernel_name.as_ptr(),
        )
    };

    assert_ne!(
        add_kernel_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let add_kernel_run_handle: xrtRunHandle = unsafe { xrtRunOpen(add_kernel_handle) };

    assert_ne!(
        add_kernel_run_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let arg: u32 = 1;

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 0, arg) }, 0,);

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 1, arg) }, 0,);

    let group_id_handle: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 2) };

    assert!(group_id_handle >= 0);

    let return_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            4,
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            group_id_handle as std::os::raw::c_uint,
        )
    };

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, group_id_handle, return_buffer_handle) },
        0,
    );

    assert_eq!(unsafe { xrtRunStart(add_kernel_run_handle) }, 0);

    assert_eq!(
        unsafe { xrtRunWait(add_kernel_run_handle) },
        ert_cmd_state_ERT_CMD_STATE_COMPLETED,
    );

    assert_eq!(
        unsafe {
            xrtBOSync(
                return_buffer_handle,
                xclBOSyncDirection_XCL_BO_SYNC_BO_FROM_DEVICE,
                4,
                0,
            )
        },
        0,
    );

    /*
    let mut result: u32 = 0;
    let result_ptr: *mut u32 = &mut result;
    assert_eq!(
        unsafe {
            xrtBORead(return_buffer_handle, result_ptr as *mut std::os::raw::c_void, 4, 0)
        },
        0,
    );

    assert_eq!(result, 2);
    */

    assert_eq! {
        unsafe {
            xrtBOFree(return_buffer_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtRunClose(add_kernel_run_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtKernelClose(add_kernel_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtDeviceClose(device_handle)
        },
        0,
    };
}

#[derive(Debug)]
pub enum XRTRSError {
    GeneralError(String),
    XclbinNotLoadedError, // For when an XCLBIN is required but not present
    NoDeviceLoadedError,
    InvalidDeviceIDError,
    UUIDRetrievalError,
    XclbinFilenameAllocError,
    XclbinLoadingError, // For when the loading of the XCLBIN itself fails 
    KernelCreationError,
}


#[allow(dead_code)]
pub struct XRTRSDevice {
    device_handle: Option<xrtDeviceHandle>,
    xclbin_handle: Option<xrtXclbinHandle>,
    xclbin_uuid: Option<xuid_t>,
    kernel_handles: HashMap<String, xrtKernelHandle>
}

#[allow(dead_code)]
impl XRTRSDevice {
    // -- Builder Interface --
    pub fn from_index(index: u32) -> Result<XRTRSDevice, XRTRSError> {
        let dh = unsafe {
            XRTRSDevice {
                device_handle: Some(xrtDeviceOpen(index)),
                xclbin_handle: None,
                xclbin_uuid: None,
                kernel_handles: HashMap::new()
            }
        };
        if dh.get_handle()? == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
            return Err(XRTRSError::InvalidDeviceIDError);
        }
        Ok(dh)
    }

    pub fn with_xclbin(mut self, filepath: &str) -> Result<XRTRSDevice, XRTRSError> {
        self.load_xclbin(filepath)?;
        Ok(self)
    }

    pub fn with_kernel(mut self, name: &str) -> Result<XRTRSDevice, XRTRSError> {
        self.load_kernel(name)?;
        Ok(self)   
    }

    // -- Methods --
    /// Opens a device with a given index. If the device handle was set before, the loaded Xclbin, UUID and kernels get deleted. Returns an error if the opening failed.
    pub fn open_device(&mut self, index: u32) -> Result<(), XRTRSError> {
        if self.device_handle.is_some() {
            self.xclbin_handle = None;
            self.xclbin_uuid = None;
            self.kernel_handles.clear();
        }
        unsafe {
            self.device_handle = Some(xrtDeviceOpen(index));
            if self.get_handle()? == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTRSError::InvalidDeviceIDError);
            } else {
                return Ok(())
            }
        }
    }

    /// Checks whether the XRTRSDevice is ready to execute kernels. This requires a loaded xclbin, a correctly initialized device and
    /// a set UUID
    pub fn is_ready(&self) -> bool {
        self.xclbin_handle.is_some() && self.xclbin_uuid.is_some() && self.device_handle.is_some()
    }


    pub fn get_handle(&self) -> Result<xrtDeviceHandle, XRTRSError>  {
        // Possible because we never even construct a faulty device handle
        match self.device_handle {
            None => Err(XRTRSError::NoDeviceLoadedError),
            Some(dh) => Ok(dh)
        }
    } 

    /// Sets the UUID of the Xclbin. This requires a valid device handle and a loaded xclbin
    fn set_uuid(&mut self) -> Result<(), XRTRSError> {
        match self.xclbin_handle {
            None => Err(XRTRSError::XclbinNotLoadedError),
            Some(xclbinhandle) => {
                let mut raw_uuid: xuid_t = [0; 16]; 
                unsafe {
                    let retval = xrtXclbinGetUUID(xclbinhandle, raw_uuid.as_mut_ptr());
                    if retval != 0 {
                        return Err(XRTRSError::UUIDRetrievalError)
                    }
                }
                self.xclbin_uuid = Some(raw_uuid);
                return Ok(());
            }
        }
    }

    /// Load the xclbin by filename and additionally set the UUID member of the XRTRSDevice.
    pub fn load_xclbin(&mut self, filepath: &str) -> Result<(), XRTRSError> {
        // 1. Alloc the xclbin filename
        // 2. Load the xclbin onto the device
        // 3. Set UUID for the loaded XCLBIN
        let cstring_path = std::ffi::CString::new(filepath)
            .expect("Failed to create cstring from given filepath!");

        let handle: xrtXclbinHandle;
        unsafe {
            handle = xrtXclbinAllocFilename(cstring_path.as_ptr() as *const i8);
            if handle == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTRSError::XclbinFilenameAllocError);
            }
        }

        unsafe {
            if xrtDeviceLoadXclbinHandle(self.device_handle.unwrap(), handle) != 0 {
                return Err(XRTRSError::XclbinLoadingError);
            }
        }
        
        // Only now set the structs handle, in case the loading failed
        self.xclbin_handle = Some(handle);

        // Set the uuid of the xclbin
        self.set_uuid()?;
        Ok(())
    }


    // Load a kernel by name. This name is then used to store it in a XRTRSDevice internal hashmap
    fn load_kernel(self: &mut XRTRSDevice, name: &str) -> Result<(), XRTRSError> {
        // If XCLBIN and UUID are set, load and store a handle to the specified kernel by it's name
        let raw_kernel_name = std::ffi::CString::new(name).expect("Error on creation of kernel name string!");
        if self.xclbin_handle == None {
            return Err(XRTRSError::XclbinNotLoadedError)
        }
        
        let kernel_handle: xrtKernelHandle;
        unsafe {
            let mut uuid = self.xclbin_uuid.ok_or(XRTRSError::GeneralError("Cannot set kernel handler for a device without an XCLBIN handler".to_string()))?;

            kernel_handle = xrtPLKernelOpen(self.device_handle.unwrap(), uuid.as_mut_ptr(), raw_kernel_name.as_ptr());
            if kernel_handle == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTRSError::KernelCreationError);
            }
        };

        self.kernel_handles.insert(name.to_string(), kernel_handle);
        Ok(())        
    }
}