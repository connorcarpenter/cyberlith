
uniform float metallic;
uniform float roughness;
uniform vec3 cameraPosition;
uniform vec4 albedo;
uniform vec4 emissive;

in vec3 pos;
in vec3 nor;

layout (location = 0) out vec4 outColor;

void main()
{
    vec4 surface_color = albedo;
    float metallic_factor = metallic;
    float roughness_factor = roughness;

    float occlusion = 1.0;

    vec3 normal = normalize(gl_FrontFacing ? nor : -nor);

    vec3 total_emissive = emissive.rgb;

    outColor.rgb = total_emissive + calculate_lighting(cameraPosition, surface_color.rgb, pos, normal, metallic_factor, roughness_factor, occlusion);
    outColor.rgb = reinhard_tone_mapping(outColor.rgb);
    outColor.rgb = srgb_from_rgb(outColor.rgb);
    outColor.a = surface_color.a;
}