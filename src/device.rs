use crate::ffi::*;

use crate::utils::is_null;
use crate::{Error, Result};

use crate::kernel::XRTKernel;

use std::collections::HashMap;

pub struct XRTDevice {
    pub(crate) handle: Option<xrtDeviceHandle>,
    pub(crate) xclbin_handle: Option<xrtXclbinHandle>,
    pub(crate) xclbin_uuid: Option<xuid_t>,
    kernels: HashMap<String, XRTKernel>,
}

impl TryFrom<u32> for XRTDevice {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self> {
        let handle = unsafe { xrtDeviceOpen(value) };
        if is_null(handle) {
            return Err(Error::DeviceOpenError);
        }
        Ok(XRTDevice {
            handle: Some(handle),
            xclbin_handle: None,
            xclbin_uuid: None,
            kernels: HashMap::new(),
        })
    }
}

impl XRTDevice {
    // TODO: constructor from PCIe bdf

    pub fn load_xclbin(&mut self, path: &str) -> Result<()> {
        if let Some(handle) = self.handle {
            let fpath_converted = match std::ffi::CString::new(path) {
                Ok(val) => val,
                Err(_) => return Err(Error::CStringCreationError),
            };
            let xclbin_handle = unsafe { xrtXclbinAllocFilename(fpath_converted.as_ptr()) };
            if is_null(xclbin_handle) {
                return Err(Error::XclbinFileAllocError);
            }
            if unsafe { xrtDeviceLoadXclbinHandle(handle, xclbin_handle) } != 0 {
                return Err(Error::XclbinLoadError);
            }
            let mut uuid: xuid_t = [0; 16];
            let retval = unsafe { xrtXclbinGetUUID(xclbin_handle, uuid.as_mut_ptr()) };
            if retval != 0 {
                return Err(Error::XclbinUUIDRetrievalError);
            }

            self.xclbin_handle = Some(xclbin_handle);
            self.xclbin_uuid = Some(uuid);
            Ok(())
        } else {
            return Err(Error::UnopenedDeviceError);
        }
    }

    pub fn with_xclbin(mut self, path: &str) -> Result<Self> {
        self.load_xclbin(path)?;
        Ok(self)
    }

    pub fn with_kernel(mut self, name: &str) -> Result<Self> {
        if !self.kernels.contains_key(name) {
            self.kernels
                .insert(name.into(), XRTKernel::new(name, &self)?);
        }
        Ok(self)
    }

    pub fn kernel(&self, name: &str) -> Result<&XRTKernel> {
        self.kernels.get(name).ok_or(Error::KernelNotLoadedYetError)
    }

    pub fn is_ready(&self) -> bool {
        self.handle.is_some() && self.xclbin_handle.is_some() && self.xclbin_uuid.is_some()
    }
}

impl Drop for XRTDevice {
    fn drop(&mut self) {
        unsafe {
            if let Some(handle) = self.xclbin_handle {
                xrtXclbinFreeHandle(handle);
            }
            if let Some(handle) = self.handle {
                xrtDeviceClose(handle);
            }
        }
    }
}
