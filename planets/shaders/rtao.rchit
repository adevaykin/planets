#version 460 core
#extension GL_EXT_ray_tracing : enable
layout(location = 0) rayPayloadInEXT vec4 payload;

void main() {
    vec3 hitWorldPos = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    float distToCamera= length(hitWorldPos - vec3(0.0, 0.0, -2.0));

    payload = vec4(vec3(distToCamera/10.0), 1.0);
}
