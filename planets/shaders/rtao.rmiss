#version 460 core
#extension GL_EXT_ray_tracing : enable

#include "rtCommon.glsl"

layout(location = 0) rayPayloadInEXT RayPayload payload;
//layout(location = 1) rayPayloadInEXT vec4 debugPayload;

void main() {
    payload.color = vec4(0.0);
    payload.aoRayMissed = true;
}
