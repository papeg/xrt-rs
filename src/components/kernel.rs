include!("../bindings_c.rs");
use crate::components::common::*;
use crate::components::device::*;

pub struct XRTKernel {
    handle: Option<xrtKernelHandle>
}

impl XRTKernel {
    pub fn new(name: &str, device: &XRTDevice) -> Result<Self, XRTError> {
        if !device.is_ready() {
            return Err(XRTError::DeviceNotReadyError);
        }

        let kernel_name = std::ffi::CString::new(name).expect("Tried creating CString from kernel name");
        let handle = unsafe { 
            xrtPLKernelOpen(device.get_handle().unwrap(), device.get_uuid().unwrap().as_mut_ptr(), kernel_name.as_ptr()) 
        };
    
        if is_null(handle) {
            return Err(XRTError::KernelCreationError);
        }

        Ok(XRTKernel { handle: Some(handle) })
    }
}