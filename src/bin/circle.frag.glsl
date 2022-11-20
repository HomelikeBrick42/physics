#version 330 core

in vec2 v_Position;
in vec2 v_TexCoord;

out vec4 o_Color;

uniform vec3 u_Color = vec3(1.0);
uniform sampler2D u_Texture;

void main() {
  o_Color = vec4(u_Color, 1.0) * texture(u_Texture, v_TexCoord);
  if (dot(v_Position, v_Position) >= 1.0) {
    discard;
  }
}
