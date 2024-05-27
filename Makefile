.PHONY: xclbin clean

TARGET := hw

xclbin: add_$(TARGET).xclbin

VPP_FLAGS := --platform $(PLATFORM) --target $(TARGET) --save-temps --debug
COMP_FLAGS := --compile $(VPP_FLAGS)
LINK_FLAGS := --link --optimize 3 $(VPP_FLAGS)

add_$(TARGET).xo: ./add.cpp
	v++ $(COMP_FLAGS) --temp_dir _x_add --kernel add --output $@ $^

add_$(TARGET).xclbin: add_$(TARGET).xo
	v++ $(LINK_FLAGS) --temp_dir _x_add_xclbin --output $@ add_$(TARGET).xo


clean:
	git clean -Xdf
