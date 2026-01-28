layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec4 in_colour;
layout(location = 2) in vec2 in_uv;

const mat4 projection = mat4(
    2, 0, 0, 0,
    0, -2, 0, 0,
    0, 0, -0.001, 0,
    -1, 1, 1, 1
);

out vec4 vert_colour;
out vec2 uv;

void main() {
    vert_colour = in_colour;
    uv = in_uv;
    gl_Position = projection * vec4(in_pos, 1.0);
}

