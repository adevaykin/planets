#ifndef CAMERA_GLSL
#define CAMERA_GLSL

layout(binding = 16) uniform CameraUBO {
    mat4 view;
    mat4 viewInverse;
    mat4 proj;
    mat4 projInverse;
    vec4 viewportExtent;
} cameraUbo;

#endif