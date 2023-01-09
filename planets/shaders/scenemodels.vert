#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "camera.glsl"
#include "timer.glsl"

layout(binding = 13) readonly buffer ModelData {
    mat4 model[1024];
} modelData;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) out vec3 fragPosition;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    mat4 modelTransform = modelData.model[gl_InstanceIndex];
    fragNormal = mat3(transpose(inverse(modelTransform))) * inNormal; // TODO: do it on the CPU side
    fragTexCoord = inTexCoord;
    fragPosition = vec3(modelTransform * vec4(inPosition, 1.0));

    gl_Position = cameraUbo.proj * cameraUbo.view * modelTransform * vec4(inPosition * timerUbo.frameTimeDelta, 1.0); // TODO: this can probably be optimized by reusing fragPosition
}
