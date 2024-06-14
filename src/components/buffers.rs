use crate::components::common::*;
use crate::components::device::*;
use crate::ffi::*;
use std::ffi::c_void;

pub enum SyncDirection {
    HostToDevice,
    DeviceToHost,
}

impl Into<xclBOSyncDirection> for SyncDirection {
    fn into(self) -> xclBOSyncDirection {
        match self {
            SyncDirection::HostToDevice => xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE,
            SyncDirection::DeviceToHost => xclBOSyncDirection_XCL_BO_SYNC_BO_FROM_DEVICE
        }
    }
}

pub struct XRTBuffer {
    handle: Option<xrtBufferHandle>,
    size: usize,
}

impl XRTBuffer {
    /// Create a new buffer. Buffers are bound to devices, but not to kernels. However if used for a kernel as an argument,
    /// the memory group must match. The memory group for a kernel arg can be retrieved via  kernel.get_memory_group_for_argument
    pub fn new(
        device: &XRTDevice,
        size: usize,
        flags: u32,
        memory_group: i32,
    ) -> Result<Self, XRTError> {
        if device.get_handle().is_none() {
            return Err(XRTError::UnopenedDeviceError);
        }
        let handle = unsafe {
            xrtBOAlloc(
                device.get_handle().unwrap(),
                size,
                flags as u64,
                memory_group as u32,
            )
        };
        if is_null(handle) {
            return Err(XRTError::BOCreationError);
        }
        Ok(XRTBuffer {
            handle: Some(handle),
            size: size
        })
    }

    pub fn get_handle(&self) -> Option<xrtBufferHandle> {
        self.handle.clone()
    }

    /// Sync the BO in the given direction. If size is given use that value, else synchronize the buffer
    pub fn sync(&self, sync_direction: SyncDirection, size: Option<usize>, seek: usize) -> Result<(), XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::BONotCreatedYet);
        }
        let used_size = match size {
            None => self.size,
            Some(s) => s
        };
        let ret_val = unsafe { xrtBOSync(self.handle.unwrap(), sync_direction.into(), used_size, seek) };

        // TODO: Implement XRT error code handling: https://github.com/Xilinx/XRT/blob/master/src/runtime_src/core/include/xrt_error_code.h (Returned by some functions to specify what kind of error ocurred) if ret_val != 0 {
        if ret_val != 0 {
            return Err(XRTError::BOSyncError);
        }
        Ok(())
    }

    /// Write the given datatype into the buffer. Buffer still needs to be synced for the data to show up on the FPGA
    pub fn write<T>(&self, data: &[T], seek: usize) -> Result<(), XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::BONotCreatedYet);
        }
        let ret_val = unsafe {
            xrtBOWrite(self.handle.unwrap(), data.as_ptr() as *const c_void, data.len(), seek)
        };

        // TODO: Implement XRT error code handling: https://github.com/Xilinx/XRT/blob/master/src/runtime_src/core/include/xrt_error_code.h (Returned by some functions to specify what kind of error ocurred) if ret_val != 0 {
        if ret_val != 0 {
            return Err(XRTError::BOWriteError);
        }
        Ok(())
    }

    /// Inplace reads value from BO into the provided slice
    pub fn read<T>(&self, data: &mut [T], seek: usize) -> Result<(), XRTError> {
        if self.handle.is_none() {
            return Err(XRTError::BONotCreatedYet);
        }
        let ret_val = unsafe {
            xrtBORead(self.handle.unwrap(), data.as_mut_ptr() as *mut c_void, data.len(), seek)
        };

        // TODO: Implement XRT error code handling: https://github.com/Xilinx/XRT/blob/master/src/runtime_src/core/include/xrt_error_code.h (Returned by some functions to specify what kind of error ocurred)
        if ret_val != 0 {
            return Err(XRTError::BOReadError);
        }
        Ok(())
    }


}

impl Drop for XRTBuffer {
    fn drop(&mut self) {
        if self.handle.is_some() {
            unsafe {
                xrtBOFree(self.handle.unwrap());
            }
            self.handle = None;
        }
    }
}
