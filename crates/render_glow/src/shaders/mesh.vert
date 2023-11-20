
uniform mat4 viewProjection;
in vec3 vertex_world_position;
in vec3 vertex_world_normal;

in vec4 transform_row1;
in vec4 transform_row2;
in vec4 transform_row3;

flat out vec4 inColor;

//

uniform float metallic;
uniform float roughness;
uniform vec3 camera_position;
uniform vec4 albedo;
uniform vec4 emissive;

void main()
{
    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);

    vec4 worldPosition = transform * vec4(vertex_world_position, 1.);
    worldPosition /= worldPosition.w;
    vec3 transformed_vertex_world_position = worldPosition.xyz;

    mat3 normalMatrix = mat3(transpose(inverse(transform)));
    vec3 transformed_vertex_world_normal = normalize(normalMatrix * vertex_world_normal);

    gl_Position = viewProjection * worldPosition;

    ///

    inColor.rgb = emissive.rgb + calculate_total_light(camera_position, albedo.rgb, transformed_vertex_world_position, transformed_vertex_world_normal, metallic, roughness);
    inColor.rgb = reinhard_tone_mapping(inColor.rgb);
    inColor.rgb = srgb_from_rgb(inColor.rgb);
    inColor.a = albedo.a;
}