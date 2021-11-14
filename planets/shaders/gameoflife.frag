#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "camera.glsl"
#include "timer.glsl"

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main()
{
    ivec2 pixelPerCell = ivec2(cameraUbo.viewportExtent.zw)/ivec2(10, 10);

    if (int(gl_FragCoord.x) % pixelPerCell.x == 0 || int(gl_FragCoord.y) % pixelPerCell.y == 0)
    {
		outColor = vec4(0.0);
        return;
    }

    ivec2 aliveCell = ivec2(3, 7);
    ivec2 aliveCellStart = aliveCell * pixelPerCell;
    ivec2 aliveCellEnd = aliveCellStart + pixelPerCell;
    if (int(gl_FragCoord.x) >= aliveCellStart.x && int(gl_FragCoord.x) <= aliveCellEnd.x)
    {
        if (int(gl_FragCoord.y) >= aliveCellStart.y && int(gl_FragCoord.y) <= aliveCellEnd.y)
        {
            outColor = vec4(0.0, 0.0, 0.0, 1.0);
            return;
        }
    }

	discard;
}