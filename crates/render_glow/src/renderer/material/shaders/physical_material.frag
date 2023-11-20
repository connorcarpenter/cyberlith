
uniform float metallic;
uniform float roughness;
uniform vec3 cameraPosition;
uniform vec4 albedo;
uniform vec4 emissive;

in vec3 pos;

layout (location = 0) out vec4 outColor;

void main()
{
    vec4 surface_color = albedo;
    float metallic_factor = metallic;
    float roughness_factor = roughness;

    vec3 dx = dFdx(pos);
    vec3 dy = dFdy(pos);
    vec3 normal = normalize(cross(dx, dy));

    vec3 total_emissive = emissive.rgb;

    outColor.rgb = total_emissive + calculate_lighting(cameraPosition, surface_color.rgb, pos, normal, metallic_factor, roughness_factor);
    outColor.rgb = reinhard_tone_mapping(outColor.rgb);
    outColor.rgb = srgb_from_rgb(outColor.rgb);
    outColor.a = surface_color.a;
}