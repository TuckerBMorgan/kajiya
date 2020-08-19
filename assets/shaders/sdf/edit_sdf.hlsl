#include "sdf_common.hlsl"

RWTexture3D<float> output_tex;

static float3 ws_pos;

float sd_sphere(float3 p, float s) {
  return length(p) - s;
}

float op_sub(float d1, float d2) {
    return max(-d1, d2);
}

float op_union(float d1, float d2) {
    return min(d1, d2);
}

[numthreads(4, 4, 4)]
void main(in uint3 pix : SV_DispatchThreadID) {
    float3 ws_pos = ((float3(pix) / float3(SDFRES, SDFRES, SDFRES)) - 0.5) * 2.0 * HSIZE;

    float result = 0.5;

    if (frame_constants.mouse.z > 0.0) {
        float3 mouse_pos = get_sdf_brush_pos();
        result = op_union(result, sd_sphere(ws_pos - mouse_pos, get_sdf_brush_radius()));
    }
    
    //result = op_union(s1, s0);

    /*if (frame_idx % 2 == 0) {
        result = op_union(s1, s0);
    }*/

    float prev = output_tex[pix];

    if (frame_constants.mouse.w > 0.0) {
        result = op_union(result, prev);
    } else {
        result = op_sub(result, prev);
    }

    output_tex[pix] = result;
}
