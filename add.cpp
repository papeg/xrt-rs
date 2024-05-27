#include <stdint.h>

extern "C" {
void add(uint32_t in_0, uint32_t in_1, uint32_t* out) {
    out[0] = in_0 + in_1;    
}
}
