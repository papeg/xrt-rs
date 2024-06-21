use crate::device::*;
use crate::ffi::*;
use crate::run::XRTRun;
use crate::utils::is_null;
use crate::{Error, Result};

pub struct XRTKernel {
    pub(crate) handle: Option<xrtKernelHandle>,
}

impl XRTKernel {
    pub fn new(name: &str, device: &XRTDevice) -> Result<Self> {
        if !device.is_ready() {
            // TODO: Maybe use XclbinNotLoadedError instead? To be more precise
            return Err(Error::DeviceNotReadyError);
        }

        if let Ok(kernel_name) = std::ffi::CString::new(name) {
            let handle = unsafe {
                xrtPLKernelOpen(
                    device.handle.unwrap(),
                    device.xclbin_uuid.unwrap().as_mut_ptr(),
                    kernel_name.as_ptr(),
                )
            };

            if is_null(handle) {
                return Err(Error::KernelCreationError);
            }

            Ok(XRTKernel {
                handle: Some(handle),
            })
        } else {
            return Err(Error::CStringCreationError);
        }
    }

    pub fn run(&self) -> Result<XRTRun> {
        XRTRun::try_from(self)
    }

    /// Get the memory group for the buffer that is used as an argument to this kernel. This is needed when creating the buffer object
    /// whoose pointer is passed to the kernel function
    pub fn get_memory_group_for_argument(&self, argno: i32) -> Result<i32> {
        if let Some(handle) = self.handle {
            let grp = unsafe { xrtKernelArgGroupId(handle, argno) };
            if grp < 0 {
                return Err(Error::KernelArgRtrvError);
            }
            Ok(grp)
        } else {
            return Err(Error::KernelNotLoadedYetError);
        }
    }
}

impl Drop for XRTKernel {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            unsafe { xrtKernelClose(handle) };
        }
    }
}
