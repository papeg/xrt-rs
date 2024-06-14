#include <stdint.h>
#include <stdio.h>

extern "C" {
void add(uint32_t in_0, uint32_t in_1, uint32_t* out) {
    printf("in_0: %u\n", in_0);
    printf("in_1: %u\n", in_1);
    out[0] = in_0 + in_1;    
    printf("out[0]: %u\n", out[0]);
}
}
