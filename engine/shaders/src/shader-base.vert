#version 450
#extension GL_ARB_separate_shader_objects : enable

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out vec3 fragColor;

const vec2 positions[6] = {
    // triangle ABC
    vec2(-1.0,  1.0),
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    // triangle ACD
    vec2(-1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2( 1.0,  1.0),
};

const vec3 colors[6] = {
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
};

const vec2 uvs[6] = {
    // triangle ABC
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    // triangle ACD
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0),
};

//layout(location = 0) out vec2 ftex;

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    //ftex = uvs[gl_VertexIndex];
    fragColor = colors[gl_VertexIndex];
}