#version 460 core
#extension GL_EXT_ray_tracing : enable
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(location = 0) rayPayloadInEXT vec4 payload;

//layout(buffer_reference, scalar) buffer Vertices { vec3 v[]; }; // Positions of an object
//layout(buffer_reference, scalar) buffer Indices { ivec3 i[]; }; // Triangle indices

// Information of a obj model when referenced in a shader
struct ObjDesc
{
    uint64_t vertexAddress;         // Address of the Vertex buffer
    uint64_t indexAddress;          // Address of the index buffer
};

layout(binding = 12, set = 0, scalar) buffer ObjDescs { ObjDesc i[]; } objDesc;

void main() {
    vec3 hitWorldPos = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    float distToCamera= length(hitWorldPos - vec3(0.0, 0.0, -2.0));

    payload = vec4(vec3(distToCamera/10.0), 1.0);
}
