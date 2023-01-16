#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "lights.glsl"

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragPosition;

layout(location = 0) out vec4 outColor;

//layout(binding = 2) uniform sampler2D texSampler;

const vec3 ambientColor = vec3(0.1, 0.1, 0.1);
const vec4 specularColor = vec4(1.0, 1.0, 1.0, 1.0);
const float specularStrength = 2.0;

void main() {
    vec3 lightContribution = vec3(0.0);

    vec3 normal = normalize(fragNormal);
    vec3 viewDir = normalize(vec3(0.0, 0.0, -10.0) - fragPosition);

//    for (int i=0; i<MAX_LIGHTS; i++)
//    {
//        Light light = lightsUbo.lights[i];
//        if (isLightActive(light))
//        {
//            vec3 lightDir = normalize(light.position.xyz - fragPosition);
//
//            float diff = max(dot(normal, lightDir), 0.0);
//            vec3 diffuse = diff * light.color.rgb;
//
//            vec3 reflectDir = reflect(-lightDir, normal);
//            float spec = pow(max(dot(viewDir, reflectDir), 0.0), 64);
//            vec3 specular = specularStrength * spec * light.color.rgb;
//
//            lightContribution += diffuse + specular;
//        }
//    }
    //outColor = vec4(fragTexCoord, 0.0, 1.0);
    vec3 lightDir = normalize(vec3(0.2, 0.2, -1.0) - fragPosition);
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = vec3(0.0);//diff * vec3(0.5);

    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 64);
    vec3 specular = specularStrength * spec * vec3(1.0, 1.0, 1.0);

    lightContribution = diffuse;//+ specular;
    outColor = vec4((ambientColor*5.0 + lightContribution)/* * texture(texSampler, fragTexCoord).rgb*/, 1.0);
}