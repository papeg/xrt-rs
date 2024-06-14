use crate::ffi::*;

use crate::{Error, Result};
use crate::utils::is_null;

pub struct XRTDevice {
    handle: Option<xrtDeviceHandle>,
    xclbin_handle: Option<xrtXclbinHandle>,
    xclbin_uuid: Option<xuid_t>,
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
    pub fn new() -> Self {
        XRTDevice {
            handle: None,
            xclbin_handle: None,
            xclbin_uuid: None,
        }
    }

    pub fn from_index(index: u32) -> Result<Self> {
        XRTDevice::try_from(index)
    }

    pub fn get_handle(&self) -> Option<xrtDeviceHandle> {
        self.handle.clone()
    }

    pub fn get_uuid(&self) -> Option<xuid_t> {
        self.xclbin_uuid.clone()
    }

    pub fn load_xclbin(&mut self, path: &str) -> Result<()> {
        if let None = self.handle {
            return Err(Error::UnopenedDeviceError);
        }
        let fpath_converted = match std::ffi::CString::new(path) {
            Ok(val) => val,
            Err(_) => return Err(Error::CStringCreationError),
        };
        let xclbin_handle = unsafe { xrtXclbinAllocFilename(fpath_converted.as_ptr()) };
        if is_null(xclbin_handle) {
            return Err(Error::XclbinFileAllocError);
        }
        if unsafe { xrtDeviceLoadXclbinHandle(self.handle.unwrap(), xclbin_handle) } != 0 {
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
    }

    pub fn is_ready(&self) -> bool {
        self.handle.is_some() && self.xclbin_handle.is_some() && self.xclbin_uuid.is_some()
    }
}

impl Drop for XRTDevice {
    fn drop(&mut self) {
        unsafe {
            if let Some(h) = self.xclbin_handle {
                xrtXclbinFreeHandle(h);
            }
            self.xclbin_handle = None;
            if let Some(h) = self.handle {
                xrtDeviceClose(h);
            }
            self.handle = None;
        }
    }
}
