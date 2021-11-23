#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "camera.glsl"
#include "timer.glsl"

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

// Must match GAME_FIELD_SIZE constant from app.rs
const uint FIELD_SIZE = 10;

layout(binding = 0) readonly buffer GameData {
    uint data[FIELD_SIZE * FIELD_SIZE];
} fieldData;

void main()
{
    ivec2 pixelPerCell = ivec2(cameraUbo.viewportExtent.zw/FIELD_SIZE);
    ivec2 currentCell = ivec2(int(gl_FragCoord.x) / pixelPerCell[0], int(gl_FragCoord.y) / pixelPerCell[1]);
    if (currentCell.x >= FIELD_SIZE || currentCell.y >= FIELD_SIZE)
    {
        discard;
    }

    if (int(gl_FragCoord.x) % pixelPerCell.x == 0 || int(gl_FragCoord.y) % pixelPerCell.y == 0)
    {
        outColor = vec4(0.0);
        return;
    }

    if (fieldData.data[currentCell[0]*FIELD_SIZE + currentCell[1]] > 0)
    {
        outColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

	discard;
}