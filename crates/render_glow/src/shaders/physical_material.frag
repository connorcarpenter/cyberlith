
uniform float metallic;
uniform float roughness;
uniform vec3 camera_position;
uniform vec4 albedo;
uniform vec4 emissive;

in vec3 vertex_view_position;

layout (location = 0) out vec4 outColor;

void main()
{
    vec4 surface_color = albedo;
    float metallic_factor = metallic;
    float roughness_factor = roughness;

    vec3 dx = dFdx(vertex_view_position);
    vec3 dy = dFdy(vertex_view_position);
    vec3 normal1 = normalize(cross(dx, dy));
    vec3 normal = vec3(normal1.x, -normal1.z, normal1.y); // ?

    vec3 total_emissive = emissive.rgb;

    outColor.rgb = total_emissive + calculate_total_light(camera_position, surface_color.rgb, vertex_view_position, normal, metallic_factor, roughness_factor);
    outColor.rgb = reinhard_tone_mapping(outColor.rgb);
    outColor.rgb = srgb_from_rgb(outColor.rgb);
    outColor.a = surface_color.a;
}