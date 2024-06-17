.PHONY: xclbin clean test memcheck

PLATFORM := xilinx_u280_gen3x16_xdma_1_202211_1
TARGET := sw_emu

VPP_FLAGS := --platform $(PLATFORM) --target $(TARGET) --save-temps --debug
COMP_FLAGS := --compile $(VPP_FLAGS)
LINK_FLAGS := --link --optimize 3 $(VPP_FLAGS)

DATATYPES = u32 u64 i32 i64 f32 f64

xclbin: $(foreach TYPE,$(DATATYPES),vscale_$(TYPE)_$(TARGET).xclbin)

define create_target_for_type
vscale_$1_$(TARGET).xo: ./vscale.cpp
	v++ $(COMP_FLAGS) --temp_dir _x_vscale_$1_$(TARGET) --kernel vscale_$1 --output vscale_$1_$(TARGET).xo vscale.cpp

vscale_$1_$(TARGET).xclbin: vscale_$1_$(TARGET).xo
	v++ $(LINK_FLAGS) --temp_dir _x_vscale_$1_$(TARGET)_xclbin --output vscale_$1_$(TARGET).xclbin vscale_$1_$(TARGET).xo
endef

$(foreach TYPE,$(DATATYPES),$(eval $(call create_target_for_type,$(TYPE))))

clean:
	git clean -Xdf
