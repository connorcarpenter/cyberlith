
uniform mat4 view_projection;
uniform vec3 camera_position;

uniform vec3 material_color;
uniform vec3 material_emissive;
uniform float material_shininess;

in vec3 vertex_world_position;
in vec3 vertex_world_normal;

in vec4 transform_row1;
in vec4 transform_row2;
in vec4 transform_row3;

flat out vec3 color;

void main()
{
    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);

    vec4 world_position = transform * vec4(vertex_world_position, 1.);
    world_position /= world_position.w;
    vec3 transformed_vertex_world_position = world_position.xyz;

    // TODO: send this via a uniform instead of calculating here!
    mat3 normal_matrix = mat3(transpose(inverse(transform)));
    vec3 transformed_vertex_world_normal = normalize(normal_matrix * vertex_world_normal);

    gl_Position = view_projection * world_position;

    ///

    color = material_emissive + calculate_total_light(camera_position, transformed_vertex_world_position, transformed_vertex_world_normal, material_color, material_shininess);
    color = reinhard_tone_mapping(color);
    color = srgb_from_rgb(color);
}