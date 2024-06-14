include!("../bindings_c.rs");
use crate::components::common::*;
use crate::components::device::*;
use std::os::raw::c_ulong;

pub struct XRTBuffer {
    handle: Option<xrtBufferHandle>,
}

impl XRTBuffer {
    /// Create a new buffer. Buffers are bound to devices, but not to kernels. However if used for a kernel as an argument,
    /// the memory group must match. The memory group for a kernel arg can be retrieved via  kernel.get_memory_group_for_argument
    pub fn new(
        device: &XRTDevice,
        size_bytes: usize,
        flags: c_ulong,
        memory_group: i32,
    ) -> Result<Self, XRTError> {
        if device.get_handle().is_none() {
            return Err(XRTError::UnopenedDeviceError);
        }
        let handle = unsafe {
            xrtBOAlloc(
                device.get_handle().unwrap(),
                size_bytes,
                flags,
                memory_group as u32,
            )
        };
        if is_null(handle) {
            return Err(XRTError::BOCreationError);
        }
        Ok(XRTBuffer {
            handle: Some(handle),
        })
    }
}

impl Drop for XRTBuffer {
    fn drop(&mut self) {
        if self.handle.is_some() {
            unsafe {
                xrtBOFree(self.handle.unwrap());
            }
        }
    }
}
