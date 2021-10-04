#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "camera.glsl"
#include "timer.glsl"

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main()
{
	outColor = vec4(gl_FragCoord.x / cameraUbo.viewportExtent.z, gl_FragCoord.y / cameraUbo.viewportExtent.w, sin(timerUbo.totalTimeElapsed), 1.0);
}