# xrt-rs
xrt-rs provides a wrapper around the C-API of the Xilinx Runtime (XRT) used for communication between AMD FPGAs / AI Engines and their host. The library offers a thin wrapper (called _native_), that simply translates the C API into safe Rust, as well as a more abstract layer (called _simple_), that automatically takes care of the details to provide an easier to use interface for simpler applications (**TODO**).

## Installation
To install simply add this repository or crate as a dependency to your `Cargo.toml`.

In case linking fails, add the XRT libs to your library path:
```
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/opt/xilinx/xrt/lib"
or
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$XILINX_XRT"
```

## Usage
*Currently* the relevant names can be imported from `xrt_rs::...`. Every concept from the XRT API has a Rust equivalent (xrtBufferHandle -> XRTBuffer for example, etc.). This may be subject to change.

## Testing
Currently the tests can not be run in parallel. 

If you want to run them in software emulation, you need to set the regular env flag.

```
XCL_EMULATION_MODE=sw_emu cargo test -- --test-threads=1
```

## TODOs
- [ ] More detailed error reporting
- [ ] Abstract layer
- [ ] Performance considerations (dont require safety checks on performance critical functions?)
- [ ] Detailed testing