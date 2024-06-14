use crate::ffi::*;
use crate::device::*;
use crate::utils::is_null;
use crate::{Result, Error};

pub struct XRTKernel {
    handle: Option<xrtKernelHandle>,
}

impl XRTKernel {
    pub fn new(name: &str, device: &XRTDevice) -> Result<Self> {
        if !device.is_ready() {
            // TODO: Maybe use XclbinNotLoadedError instead? To be more precise
            return Err(Error::DeviceNotReadyError);
        }

        let kernel_name =
            std::ffi::CString::new(name).expect("Tried creating CString from kernel name");
        let handle = unsafe {
            xrtPLKernelOpen(
                device.get_handle().unwrap(),
                device.get_uuid().unwrap().as_mut_ptr(),
                kernel_name.as_ptr(),
            )
        };

        if is_null(handle) {
            return Err(Error::KernelCreationError);
        }

        Ok(XRTKernel {
            handle: Some(handle),
        })
    }

    /// Get the memory group for the buffer that is used as an argument to this kernel. This is needed when creating the buffer object
    /// whoose pointer is passed to the kernel function
    pub fn get_memory_group_for_argument(&self, argument_number: u32) -> Result<i32> {
        if self.handle.is_none() {
            return Err(Error::KernelNotLoadedYetError);
        }
        let grp = unsafe { xrtKernelArgGroupId(self.handle.unwrap(), argument_number as i32) };
        if grp < 0 {
            return Err(Error::KernelArgRtrvError);
        }
        Ok(grp)
    }

    pub fn get_handle(&self) -> Option<xrtKernelHandle> {
        self.handle.clone()
    }
}

impl Drop for XRTKernel {
    fn drop(&mut self) {
        if self.handle.is_some() {
            unsafe { xrtKernelClose(self.handle.unwrap()) };
            self.handle = None;
        }
    }
}
