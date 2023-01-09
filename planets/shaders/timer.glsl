#ifndef TIME_GLSL
#define TIME_GLSL

layout(binding = 15) uniform TimerUBO {
    float totalTimeElapsed;
    float frameTimeDelta;
} timerUbo;

#endif