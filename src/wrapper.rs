
include!("bindings_c.rs");
use std::collections::HashMap;

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
}


#[allow(dead_code)]
pub struct XRTDevice {
    device_handle: Option<xrtDeviceHandle>,
    xclbin_handle: Option<xrtXclbinHandle>,
    xclbin_uuid: Option<xuid_t>,
    kernel_handles: HashMap<String, xrtKernelHandle>
}

// TODO: Make generic for all types that implement Num Trait
impl TryFrom<u32> for XRTDevice {
    type Error = XRTError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let dh = unsafe {
            XRTDevice {
                device_handle: Some(xrtDeviceOpen(value)),
                xclbin_handle: None,
                xclbin_uuid: None,
                kernel_handles: HashMap::new()
            }
        };
        if dh.get_handle()? == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
            return Err(XRTError::InvalidDeviceIDError);
        }
        Ok(dh)
    }
}

#[allow(dead_code)]
impl XRTDevice {
    // -- Builder Interface --
    pub fn from_index(index: u32) -> Result<XRTDevice, XRTError> {
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
        unsafe {
            self.device_handle = Some(xrtDeviceOpen(index));
            if self.get_handle()? == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTError::InvalidDeviceIDError);
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


    pub fn get_handle(&self) -> Result<xrtDeviceHandle, XRTError>  {
        // Possible because we never even construct a faulty device handle
        match self.device_handle {
            None => Err(XRTError::NoDeviceLoadedError),
            Some(dh) => Ok(dh)
        }
    } 

    /// Sets the UUID of the Xclbin. This requires a valid device handle and a loaded xclbin
    fn set_uuid(&mut self) -> Result<(), XRTError> {
        match self.xclbin_handle {
            None => Err(XRTError::XclbinNotLoadedError),
            Some(xclbinhandle) => {
                let mut raw_uuid: xuid_t = [0; 16]; 
                unsafe {
                    let retval = xrtXclbinGetUUID(xclbinhandle, raw_uuid.as_mut_ptr());
                    if retval != 0 {
                        return Err(XRTError::UUIDRetrievalError)
                    }
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

        let handle: xrtXclbinHandle;
        unsafe {
            handle = xrtXclbinAllocFilename(cstring_path.as_ptr() as *const i8);
            if handle == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTError::XclbinFilenameAllocError);
            }
        }

        unsafe {
            if xrtDeviceLoadXclbinHandle(self.device_handle.unwrap(), handle) != 0 {
                return Err(XRTError::XclbinLoadingError);
            }
        }
        
        // Only now set the structs handle, in case the loading failed
        self.xclbin_handle = Some(handle);

        // Set the uuid of the xclbin
        self.set_uuid()?;
        Ok(())
    }


    // Load a kernel by name. This name is then used to store it in a XRTRSDevice internal hashmap
    fn load_kernel(self: &mut XRTDevice, name: &str) -> Result<(), XRTError> {
        // If XCLBIN and UUID are set, load and store a handle to the specified kernel by it's name
        let raw_kernel_name = std::ffi::CString::new(name).expect("Error on creation of kernel name string!");
        if self.xclbin_handle == None {
            return Err(XRTError::XclbinNotLoadedError)
        }
        
        let kernel_handle: xrtKernelHandle;
        unsafe {
            let mut uuid = self.xclbin_uuid.ok_or(XRTError::GeneralError("Cannot set kernel handler for a device without an XCLBIN handler".to_string()))?;

            kernel_handle = xrtPLKernelOpen(self.device_handle.unwrap(), uuid.as_mut_ptr(), raw_kernel_name.as_ptr());
            if kernel_handle == std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void {
                return Err(XRTError::KernelCreationError);
            }
        };

        self.kernel_handles.insert(name.to_string(), kernel_handle);
        Ok(())        
    }
}


impl Drop for XRTDevice {
    fn drop(&mut self) {
        // TODO: Deallocate any buffers
        unsafe {
            for kernel in self.kernel_handles.values() {
                xrtKernelClose(*kernel);
            }
            xrtDeviceClose(self.device_handle.unwrap());
        }
    }
}




// Tests
#[test]
fn emu_open_device_test() {
    let mut device = XRTDevice::from_index(0).unwrap();
    assert!(device.device_handle.is_some());
}

#[test]
fn emu_open_device_load_xclbin_test() {
    let mut device = XRTDevice::from_index(0).unwrap();
    assert!(device.device_handle.is_some());
    let xclbin = "../add_hw.xclbin";
    device.load_xclbin(xclbin).unwrap();
    assert!(device.xclbin_handle.is_some());
    assert!(device.xclbin_uuid.is_some());
}