
uniform mat4 view_projection;
uniform vec3 camera_position;

uniform float transform_texture_width;
uniform float transform_texture_height;
uniform sampler2D transform_texture;

uniform vec3 material_color;
uniform vec3 material_emissive;
uniform float material_shininess;

in vec3 vertex_world_position;
in vec3 vertex_world_normal;

flat out vec3 color;

vec4 get_transform(float row_index) {
    float x_coord_i = (float(gl_InstanceID) * 3.0) + float(row_index) + 0.5;
    float y_coord_i = float(gl_DrawID) + 0.5;

    float x_coord_f = x_coord_i / transform_texture_width;
    float y_coord_f = y_coord_i / transform_texture_height;

    vec4 result = texture(
        transform_texture,
        vec2(
            x_coord_f,
            y_coord_f
        )
    );

    return result;
//    return vec4(0.0, 0.0, 0.0, 0.0);
}

void main()
{

    vec4 transform_row1 = get_transform(float(0));
    vec4 transform_row2 = get_transform(float(1));
    vec4 transform_row3 = get_transform(float(2));

    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);

    vec4 world_position = transform * vec4(vertex_world_position, 1.);
    world_position /= world_position.w;
    vec3 transformed_vertex_world_position = world_position.xyz;


    // TODO: send this via a uniform instead of calculating here?
    mat3 normal_matrix = mat3(transpose(inverse(transform)));
    vec3 transformed_vertex_world_normal = normalize(normal_matrix * vertex_world_normal);

    gl_Position = view_projection * world_position;

//    vec3 transformed_vertex_world_position = vertex_world_position;
//    vec3 transformed_vertex_world_normal = vertex_world_normal;
//    gl_Position = view_projection * vec4(vertex_world_position, 1.0);


    ///

    color = material_emissive + calculate_total_light(camera_position, transformed_vertex_world_position, transformed_vertex_world_normal, material_color, material_shininess);
    color = reinhard_tone_mapping(color);
    color = srgb_from_rgb(color);
}