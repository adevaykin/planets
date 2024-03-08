#version 460 core
#extension GL_EXT_ray_tracing : enable
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#include "rtCommon.glsl"

hitAttributeEXT vec2 attribs;

// Information of a obj model when referenced in a shader
struct ObjDesc
{
    uint64_t vertexAddress;         // Address of the Vertex buffer
    uint64_t indexAddress;          // Address of the index buffer
};

layout(location = 0) rayPayloadInEXT RayPayload payload;
//layout(location = 1) rayPayloadInEXT vec4 debugPayload;

layout(binding = 0, set = 0) uniform accelerationStructureEXT acc;
layout(binding = 2, set = 0) uniform RayParams
{
    vec4 rayOrigin;
    vec4 rayDir;
    uint sbtOffset;
    uint sbtStride;
    uint missIndex;
    float randomSeed;
} params;

layout(buffer_reference, scalar) buffer Vertices { float v[]; }; // Positions of an object
layout(buffer_reference, scalar) buffer Indices { ivec3 i[]; }; // Triangle indices
layout(binding = 12, set = 0, scalar) buffer ObjDescs { ObjDesc i[]; } objDesc;

vec2 concentricSampleDisk(vec2 u) {
    vec2 uOffset = u * 2.0 - 1.0;
    if (uOffset.x == 0.0 && uOffset.y == 0.0) return vec2(0.0);

    float theta, r;
    if (abs(uOffset.x) > abs(uOffset.y)) {
        r = uOffset.x;
        theta = 0.25 * 3.14159265359 * (uOffset.y / uOffset.x);
    } else {
        r = uOffset.y;
        theta = 0.5 * 3.14159265359 - 0.25 * 3.14159265359 * (uOffset.x / uOffset.y);
    }

    return r * vec2(cos(theta), sin(theta));
}

vec3 cosHemisphereSampling(vec2 u) {
    vec2 d = concentricSampleDisk(u);
    float z = sqrt(max(0.0, 1.0 - d.x * d.x - d.y * d.y));
    return vec3(d.x, d.y, z);
}

float PHI = 1.61803398874989484820459;  // Φ = Golden Ratio

float gold_noise(in vec2 xy, in float seed){
    return fract(tan(distance(xy*PHI, xy)*seed)*xy.x);
}

vec3 rotateVectorToSurface(vec3 inp, vec3 normal) {
    vec3 tangent = normalize(cross(normal, vec3(0.0, 1.0, 0.0)));
    vec3 bitangent = normalize(cross(normal, tangent));
    return normalize(inp.x * tangent + inp.y * bitangent + inp.z * normal);
}

void main() {
    ObjDesc objResource = objDesc.i[gl_InstanceCustomIndexEXT];
    Indices indices = Indices(objResource.indexAddress);
    Vertices vertices = Vertices(objResource.vertexAddress);

    ivec3 ind = indices.i[gl_PrimitiveID];
    vec3 floatInd = vec3(ind);

    const int vertexDataSize = 3 + 3 + 2; // Position, Normal, UV
    const int v0idx = ind.x * vertexDataSize;
    const vec3 inpPos1 = vec3(vertices.v[v0idx], vertices.v[v0idx+1], vertices.v[v0idx+2]);
    const vec3 inpNrm1 = vec3(vertices.v[v0idx+3], vertices.v[v0idx+4], vertices.v[v0idx+5]);
    const vec2 inpUv1 = vec2(vertices.v[v0idx+6], vertices.v[v0idx+7]);

    const int v1idx = ind.y * vertexDataSize;
    const vec3 inpPos2 = vec3(vertices.v[v1idx], vertices.v[v1idx+1], vertices.v[v1idx+2]);
    const vec3 inpNrm2 = vec3(vertices.v[v1idx+3], vertices.v[v1idx+4], vertices.v[v1idx+5]);
    const vec2 inpUv2 = vec2(vertices.v[v1idx+6], vertices.v[v1idx+7]);

    const int v2idx = ind.z * vertexDataSize;
    const vec3 inpPos3 = vec3(vertices.v[v2idx], vertices.v[v2idx+1], vertices.v[v2idx+2]);
    const vec3 inpNrm3 = vec3(vertices.v[v2idx+3], vertices.v[v2idx+4], vertices.v[v2idx+5]);
    const vec2 inpUv3 = vec2(vertices.v[v2idx+6], vertices.v[v2idx+7]);

    vec3 barycentrics = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);

    const vec3 pos = inpPos1 * barycentrics.x + inpPos2 * barycentrics.y + inpPos3 * barycentrics.z;
    const vec3 worldPos = vec3(gl_ObjectToWorldEXT * vec4(pos, 1.0));  // Transforming the position to world space

    const vec2 uv = inpUv1 * barycentrics.x + inpUv2 * barycentrics.y + inpUv3 * barycentrics.z;

    const vec3 nrm = normalize(inpNrm1 * barycentrics.x + inpNrm2 * barycentrics.y + inpNrm3 * barycentrics.z);
    const vec3 worldNrm = normalize(vec3(gl_WorldToObjectEXT * vec4(nrm, 1.0)));  // Transforming the normal to world space

    vec3 hitWorldPos = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    float distToCamera= length(hitWorldPos - vec3(0.0, 0.0, -2.0));

    float frontFacing = dot(-gl_WorldRayDirectionEXT, worldNrm);
    vec3 aoRayOrigin = worldPos + sign(frontFacing) * worldNrm * 0.001;
    uint rayFlags = gl_RayFlagsTerminateOnFirstHitEXT | gl_RayFlagsSkipClosestHitShaderEXT;
    uint sbtOffset = 0;
    uint sbtStride = 0;
    uint missIndex = 0;
    int aoRays = 8;
    int aoHits = 0;
    for (int i=1; i < aoRays; i++) {
        payload.aoRayMissed = false;

        vec3 rotatedSampleDir = vec3(0.0);
        vec2 u = vec2(gold_noise(uv, float(i)*params.randomSeed), gold_noise(uv, float(i+1)*params.randomSeed));
        vec3 sampleDir = cosHemisphereSampling(u);
        rotatedSampleDir = rotateVectorToSurface(sampleDir, nrm);
        traceRayEXT(acc, rayFlags, 0xff, sbtOffset, sbtStride, missIndex, aoRayOrigin, 0.001, rotatedSampleDir, 0.6f, 0);

        if (!payload.aoRayMissed) {
            aoHits++;
        }
    }

    payload.color = vec4(vec3(10.0)/10.0 * (1.0 - float(aoHits)/float(aoRays)), 1.0);
    //payload.color = vec4(vec3(distToCamera)/10.0, 1.0);
}
