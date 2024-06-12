use crate::components::common::{Argument, ArgumentIndex, ArgumentType, ERTCommandState, XRTError};
use crate::ffi::*;
use std::{collections::HashMap, os::raw::c_void};

/// Struct to manage runs. Creating a run does not start it. The current state of a given run can be checked on.
/// **Key idea** is to give the data to synchronize directly to a run instead of the buffers. When the run is started it can automatically
/// transfer the data. The user can advise it to do so at any point as well
pub struct XRTRun<'a> {
    run_handle: xrtRunHandle,
    argument_mapping: &'a HashMap<ArgumentIndex, ArgumentType>, // What kind of arguments and where to sync to
    argument_data: HashMap<ArgumentIndex, Argument>, // The content of the arguments itself
}

/// This impl does not contain a constructor because a valid run can and should only be constructed from a device!
impl<'a> XRTRun<'a> {
    /// *Do not create manually, use `XRTKernel::create_run` instead*
    pub fn new(
        rhdl: *mut c_void,
        argument_mapping: &'a HashMap<ArgumentIndex, ArgumentType>,
        argument_data: HashMap<ArgumentIndex, Argument>,
    ) -> Self {
        XRTRun {
            run_handle: rhdl,
            argument_mapping: argument_mapping,
            argument_data: argument_data,
        }
    }

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

        for (index, value) in &self.argument_data {
            match value {
                Argument::Direct(data) => {
                    let result = unsafe { xrtRunSetArg(self.run_handle, *index as i32) };
                    if result != 0 {
                        return Err(XRTError::RunArgumentSetError(*index, *data));
                    }
                }

                Argument::BufferContent(data) => {
                    // Get the buffer handle
                    let buffer_handle_result = self
                        .argument_mapping
                        .get(&index)
                        .ok_or(XRTError::InvalidArgumentIndex)?;
                    let buffer_handle = match buffer_handle_result {
                        ArgumentType::InputBuffer(bhdl) => *bhdl,
                        _ => return Err(XRTError::ExpectedInputBufferArgumentType),
                    };

                    // Write data to buffer
                    let write_result = unsafe {
                        xrtBOWrite(
                            buffer_handle,
                            data.as_ptr() as *mut std::os::raw::c_void,
                            data.len(),
                            0,
                        )
                    };
                    if write_result != 0 {
                        return Err(XRTError::FailedBOWrite);
                    }

                    // Sync to FPGA
                    let sync_result = unsafe {
                        xrtBOSync(
                            buffer_handle,
                            xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE,
                            data.len(),
                            0,
                        )
                    };
                    if sync_result != 0 {
                        return Err(XRTError::FailedBOSyncToDevice);
                    }
                }
            }
        }
        Ok(())
    }

    /// Start this run. If given wait until the result is returned. If `load_arguments` is true, the arguments are first loaded
    /// and written to the buffer and synced. This can be set to false, if the user loaded the argument manually first
    pub fn start_run(&self, load_arguments: bool, wait: bool) -> Result<ERTCommandState, XRTError> {
        if load_arguments {
            self.load_arguments()?;
        }

        if unsafe { xrtRunStart(self.run_handle) } != 0 {
            return Err(XRTError::FailedRunStart);
        }

        if wait {
            let finished_state = unsafe { xrtRunWait(self.run_handle) };
            Ok(ERTCommandState::from(finished_state))
        } else {
            Ok(self.get_state())
        }
    }
}

impl<'a> Drop for XRTRun<'a> {
    fn drop(&mut self) {
        unsafe {
            xrtRunClose(self.run_handle);
        }
        // TODO: How to handle if this fails?
    }
}
