
uniform mat4 view_projection;
uniform vec3 camera_position;

//uniform float instance_texture_width;
//uniform float instance_texture_height;
uniform sampler2D instance_texture;
uniform sampler2D material_texture;

in vec3 vertex_world_position;
in vec3 vertex_world_normal;

flat out vec3 color;

vec4 get_transform(int row_index) {
    highp int x_coord_i = (gl_InstanceID * 4) + row_index;
    highp int y_coord_i = gl_DrawID;

    vec4 result = texelFetch(
        instance_texture,
        ivec2(
            x_coord_i,
            y_coord_i
        ),
        0
    );

    return result;
}

vec4 get_material_data(int material_index, int row_index) {
    highp int x_coord_i = (material_index * 2) + row_index;

    vec4 result = texelFetch(
        material_texture,
        ivec2(
            x_coord_i,
            0
        ),
        0
    );

    return result;
}

void main()
{
    vec4 transform_row1 = get_transform(0);
    vec4 transform_row2 = get_transform(1);
    vec4 transform_row3 = get_transform(2);
    vec4 instance_data = get_transform(3);

    // transform
    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);

    // position
    vec4 world_position = transform * vec4(vertex_world_position, 1.0);
    world_position /= world_position.w;
    vec3 transformed_vertex_world_position = world_position.xyz;
    gl_Position = view_projection * world_position;

    // normal
    // TODO: send this via a uniform instead of calculating here?
    mat3 normal_matrix = mat3(transpose(inverse(transform)));
    vec3 transformed_vertex_world_normal = normalize(normal_matrix * vertex_world_normal);

    // material
    highp int material_index = int(instance_data.x);
    vec4 material_data1 = get_material_data(material_index, 0);
    vec4 material_data2 = get_material_data(material_index, 1);
    vec3 material_color = vec3(material_data1.x, material_data1.y, material_data1.z);
    float material_emissive = material_data1.w;
    float material_shininess = material_data2.x;

    // color
    color = (material_emissive * material_color) + calculate_total_light(camera_position, transformed_vertex_world_position, transformed_vertex_world_normal, material_color, material_shininess);
    color = reinhard_tone_mapping(color);
    color = srgb_from_rgb(color);
}