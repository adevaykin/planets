#ifndef LIGHTS_GLSL
#define LIGHTS_GLSL

const int MAX_LIGHTS = 64;

struct Light {
    vec4 position;
    vec4 color;
    vec4 isActiveRadiusPadding;
};

bool isLightActive(Light light) {
    return light.isActiveRadiusPadding.x > 0.0;
}

float getLightRadius(Light light) {
    return light.isActiveRadiusPadding.y;
}

layout(binding = 14, std140) uniform LightsUBO {
    Light lights[MAX_LIGHTS];
} lightsUbo;

#endif