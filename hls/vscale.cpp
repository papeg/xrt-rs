#include <stdint.h>

template <typename T>
void vscale(uint32_t size, T scale, T *in, T *out)
{
    for (uint32_t i = 0; i < size; i++) {
        out[i] = scale * in[i];
    }
}

extern "C" {
void vscale_u32(uint32_t size, uint32_t scale, uint32_t *in, uint32_t *out)
{
    vscale<uint32_t>(size, scale, in, out);
}
void vscale_i32(uint32_t size, int32_t scale, int32_t *in, int32_t *out)
{
    vscale<int32_t>(size, scale, in, out);
}
void vscale_u64(uint32_t size, uint64_t scale, uint64_t *in, uint64_t *out)
{
    vscale<uint64_t>(size, scale, in, out);
}
void vscale_i64(uint32_t size, int64_t scale, int64_t *in, int64_t *out)
{
    vscale<int64_t>(size, scale, in, out);
}
void vscale_f32(uint32_t size, float scale, float *in, float *out)
{
    vscale<float>(size, scale, in, out);
}
void vscale_f64(uint32_t size, double scale, double *in, double *out)
{
    vscale<double>(size, scale, in, out);
}
}