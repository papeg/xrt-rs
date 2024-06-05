# xrt-rs
In case linking fails, add the XRT libs to your library path:
```
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/opt/xilinx/xrt/lib"
or
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$XILINX_XRT"
```

## Testing

Currently the tests can not be run in parallel. 

If you want to run them in software emulation, you need to set the regular env flag.

```
XCL_EMULATION_MODE=sw_emu cargo test -- --test-threads=1
```
