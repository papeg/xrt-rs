include!("../bindings_c.rs");
use std::{collections::HashMap, hash::Hash, os::raw::c_void, rc::*};
use crate::components::common::*;


/// Struct to manage runs. Creating a run does not start it. The current state of a given run can be checked on.
/// **Key idea** is to give the data to synchronize directly to a run instead of the buffers. When the run is started it can automatically
/// transfer the data. The user can advise it to do so at any point as well
pub struct XRTRun<'a> {
    run_handle: xrtRunHandle,
    argument_mapping: &'a HashMap<ArgumentIndex, ArgumentType>,   // What kind of arguments and where to sync to
    argument_data: HashMap<ArgumentIndex, Argument>               // The content of the arguments itself
}

/// This impl does not contain a constructor because a valid run can and should only be constructed from a device!
impl<'a> XRTRun<'a> {
    /// Return the current state of the run
    pub fn get_state(&self) -> ERTCommandState {
        let state: u32 = unsafe { xrtRunState(self.run_handle) };
        ERTCommandState::from(state)
    }

    /// This sets the direct arguments of the run and fills the buffers with the data and syncs them. If this is done without
    /// executing the kernel, another XRTRun might do the same and overwrite the data. 
    /// 
    /// __This is manually called by `start_run()`. So in many cases you will want to use that convenience method and
    /// only use this one in case you need to have more precise control.__
    pub fn load_arguments(&self) -> Result<(), XRTError> {
        if self.argument_data.len() != self.argument_mapping.len() {
            return Err(XRTError::NonMatchingArgumentLists);
        }

        for (index, value) in self.argument_data {
            match value {
                Argument::Direct(data) => { 
                    let result = unsafe { xrtRunSetArg(self.run_handle, index as i32) };
                    if result != 0 {
                        return Err(XRTError::RunArgumentSetError(index, data));
                    }
                },

                Argument::BufferContent(data) =>  {
                    // Get the buffer handle
                    let buffer_handle_result = self.argument_mapping.get(&index).ok_or(XRTError::InvalidArgumentIndex)?;
                    let buffer_handle = match buffer_handle_result {
                        ArgumentType::Buffer(bhdl) => *bhdl,
                        _ => return Err(XRTError::ExpectedBufferArgumentType)
                    };

                    // Write data to fpga
                    let write_result = unsafe { xrtBOWrite(buffer_handle, data.as_ptr() as *mut std::os::raw::c_void, data.len(), 0) };
                    if write_result != 0 {
                        return Err(XRTError::FailedBOWrite);
                    }

                    // Sync to FPGA
                    let sync_result = unsafe { xrtBOSync(buffer_handle, xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE, data.len(), 0) };
                    if sync_result != 0 {
                        return Err(XRTError::FailedBOSyncToDevice);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn start_run() {}
}