#version 460 core
#extension GL_EXT_ray_tracing : enable
layout(location = 0) rayPayloadInEXT vec4 payload[2];
//layout(location = 1) rayPayloadInEXT vec4 debugPayload;

void main() {
    payload[0] = vec4(0.0, 0.0, 0.0, 1.0);
    payload[1] = vec4(0.0, 0.0, 0.0, 0.0);
}
