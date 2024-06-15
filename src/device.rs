use crate::ffi::*;

use crate::utils::is_null;
use crate::{Error, Result};

pub struct XRTDevice {
    pub(crate) handle: Option<xrtDeviceHandle>,
    pub(crate) xclbin_handle: Option<xrtXclbinHandle>,
    pub(crate) xclbin_uuid: Option<xuid_t>,
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
