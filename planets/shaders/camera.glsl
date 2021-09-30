#ifndef CAMERA_GLSL
#define CAMERA_GLSL

layout(binding = 16) uniform CameraUBO {
    mat4 view;
    mat4 proj;
    vec4 viewportExtent;
} cameraUbo;

#endif