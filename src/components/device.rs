use crate::ffi::*;
use std::{collections::HashMap, rc::Rc};

use crate::components::common::{is_null, ArgumentIndex, ArgumentType, IOMode, XRTError};
use crate::components::kernel::XRTKernel;
use crate::components::run::XRTRun;

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

    pub fn with_xclbin(mut self, filepath: String) -> Result<XRTDevice<'a>, XRTError> {
        self.load_xclbin(filepath)?;
        Ok(self)
    }

    pub fn with_kernel(
        mut self,
        name: String,
        argument_mapping: HashMap<ArgumentIndex, ArgumentType>,
    ) -> Result<XRTDevice<'a>, XRTError> {
        self.load_kernel(name, argument_mapping)?;
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
    pub fn load_xclbin(&mut self, filepath: String) -> Result<(), XRTError> {
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

    pub fn get_kernel(&self, name: &str) -> Option<&XRTKernel> {
        self.kernel_handles.get(name)
    }

    /// Load a kernel by name. This name is then used to store it in a XRTRSDevice internal hashmap
    pub fn load_kernel(
        &mut self,
        name: String,
        initial_argument_mapping: HashMap<ArgumentIndex, ArgumentType>,
    ) -> Result<(), XRTError> {
        // If XCLBIN and UUID are set, load and store a handle to the specified kernel by it's name
        let raw_kernel_name =
            std::ffi::CString::new(name.clone()).expect("Error on creation of kernel name string!");
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
        for (k, v) in initial_argument_mapping {
            if let ArgumentType::NotRealizedBuffer(required_size, iomode) = v {
                let group_id_handle = unsafe { xrtKernelArgGroupId(kernel_handle, k as i32) };

                if group_id_handle < 0 {
                    return Err(XRTError::InvalidGroupIDError);
                }

                let bo_handle = unsafe {
                    xrtBOAlloc(
                        self.device_handle.unwrap(),
                        required_size as usize,
                        XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
                        group_id_handle as std::os::raw::c_uint,
                    )
                };

                if is_null(bo_handle) {
                    return Err(XRTError::FailedBOAllocError);
                }

                if let IOMode::Input = iomode {
                    argument_mapping.insert(k, ArgumentType::InputBuffer(bo_handle));
                } else if let IOMode::Output = iomode {
                    argument_mapping.insert(k, ArgumentType::OutputBuffer(bo_handle));
                }
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

        // Close kernels
        //? Necessary or automatic? Run <- Kernel <- Device
        //for kernel in self.kernel_handles.into_values() {
        //    std::mem::drop(kernel);
        // }

        // Make sure to not try to close a non-open device
        if self.device_handle.is_some() {
            unsafe { xrtDeviceClose(self.device_handle.unwrap()) };
        }
    }
}
