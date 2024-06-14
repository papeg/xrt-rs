include!("../bindings_c.rs");
use crate::components::common::*;
use crate::components::device::*;

pub struct XRTRun {
    handle: Option<xrtRunHandle>
}

impl XRTRun {

}

impl Drop for XRTRun {
    fn drop(&mut self) {
        if self.handle.is_some() {
            unsafe {
                xrtRunClose(self.handle.unwrap());
            }
        }
    }
}