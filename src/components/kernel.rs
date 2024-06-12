include!("../bindings_c.rs");
use crate::components::common::*;
use crate::components::run::*;
use std::{collections::HashMap, hash::Hash};

pub struct XRTKernel {
    kernel_handle: xrtKernelHandle,

    /// A mapping to describe how a kernel has to be called. For every argument index specifies whether a buffer handle is used (which is prepared at
    /// construction time of the XRTKernel) or whether it is left blank and requires input from XRT when calling the kernel. This is also
    /// the reason why this struct doenst need to save the buffer handles explicitly
    argument_mapping: HashMap<ArgumentIndex, ArgumentType>,
}

impl XRTKernel {
    /// Construct a new XRTKernel. This guards against acidentally not having initialized all required buffers
    pub fn new(
        kernel_handle: xrtKernelHandle,
        argument_mapping: HashMap<ArgumentIndex, ArgumentType>,
    ) -> Result<Self, XRTError> {
        if !XRTKernel::is_ready(&argument_mapping) {
            return Err(XRTError::UnrealizedBufferError);
        }
        Ok(XRTKernel {
            kernel_handle: kernel_handle,
            argument_mapping: argument_mapping,
        })
    }

    /// Tells whether the XRTKernel is ready for execution. If not, its argument mapping has to be edited
    pub fn is_ready(argument_mapping: &HashMap<ArgumentIndex, ArgumentType>) -> bool {
        !argument_mapping
            .values()
            .any(|x| matches!(x, ArgumentType::NotRealizedBuffer(_, _)))
    }

    // Create a run. Doing this does not execute any action. It just prepares the arguments
    pub fn create_run(
        &self,
        argument_data: HashMap<ArgumentIndex, Argument>,
    ) -> Result<XRTRun, XRTError> {
        if argument_data.len() != self.argument_mapping.len() {
            return Err(XRTError::NonMatchingArgumentLists);
        }
        let run_handle = unsafe { xrtRunOpen(self.kernel_handle) };
        if is_null(run_handle) {
            return Err(XRTError::RunCreationError);
        }
        Ok(XRTRun::new(
            run_handle,
            &self.argument_mapping,
            argument_data,
        ))
    }
}

impl Drop for XRTKernel {
    fn drop(&mut self) {
        // Deallocate all buffers
        for arg in self.argument_mapping.values() {
            match arg {
                ArgumentType::InputBuffer(bhdl) | ArgumentType::OutputBuffer(bhdl) => unsafe {
                    xrtBOFree(*bhdl);
                },
                _ => (),
            }
        }

        // Close kernel itself
        unsafe {
            xrtKernelClose(self.kernel_handle);
        }
        // TODO: What to do if this fails?
    }
}
