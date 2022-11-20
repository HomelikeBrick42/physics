#version 330 core

in vec2 a_Position;

out vec2 v_Position;
out vec2 v_TexCoord;

uniform mat4 u_ProjectionMatrix = mat4(1.0);
uniform mat4 u_ViewMatrix = mat4(1.0);
uniform mat4 u_ModelMatrix = mat4(1.0);

void main() {
  v_TexCoord = vec2((gl_VertexID >> 0) & 1, (gl_VertexID >> 1) & 1);
  v_Position = v_TexCoord * 2.0 - 1.0;
  gl_Position = u_ProjectionMatrix * inverse(u_ViewMatrix) * u_ModelMatrix *
                vec4(v_Position, 0.0, 1.0);
}
