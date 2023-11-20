
uniform mat4 viewProjection;
in vec3 vert_position;

in vec4 transform_row1;
in vec4 transform_row2;
in vec4 transform_row3;

out vec3 frag_position;

void main()
{
    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);

    vec4 worldPosition = transform * vec4(vert_position, 1.);
    worldPosition /= worldPosition.w;
    gl_Position = viewProjection * worldPosition;

    frag_position = worldPosition.xyz;
}