#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

// input is a struct ColoredVertex
layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 color;

layout (location = 0) out vec4 o_color;
void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * pos;
    // debugPrintfEXT("Here's a vector of floats %1.2v4f\n", pos);
    o_color = color;
}
