#version 460 core
#extension GL_EXT_ray_tracing : enable
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

hitAttributeEXT vec2 attribs;

struct Vertex
{
    float posX;
    float posY;
    float posZ;
    float normalX;
    float normalY;
    float normalZ;
    float uvX;
    float uvY;
};

// Information of a obj model when referenced in a shader
struct ObjDesc
{
    uint64_t vertexAddress;         // Address of the Vertex buffer
    uint64_t indexAddress;          // Address of the index buffer
};

layout(location = 0) rayPayloadInEXT vec4 payload[2];
//layout(location = 1) rayPayloadInEXT vec4 debugPayload;

layout(buffer_reference, scalar) buffer Vertices { float v[]; }; // Positions of an object
layout(buffer_reference, scalar) buffer Indices { ivec3 i[]; }; // Triangle indices
layout(binding = 12, set = 0, scalar) buffer ObjDescs { ObjDesc i[]; } objDesc;

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

    //payload = vec4(vec3(distToCamera/10.0), 1.0);
    payload[0] = vec4(nrm, 1.0);
    payload[1] = vec4(pos, 0.0);
}
