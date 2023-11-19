
uniform float metallic;
uniform float roughness;
uniform vec3 cameraPosition;
uniform vec4 albedo;
uniform vec4 emissive;

in vec3 pos;
flat in mat3 normalMat;

layout (location = 0) out vec4 outColor;

void main()
{
    vec4 surface_color = albedo;
    float metallic_factor = metallic;
    float roughness_factor = roughness;

    float occlusion = 1.0;

    vec3 dx = dFdx(pos);
    vec3 dy = dFdy(pos);
    vec3 normal1 = normalize(cross(dx, dy));
    vec3 normal2 = normalize(normalMat * normal1);
    vec3 normal3 = normalize(gl_FrontFacing ? normal2 : -normal2);

    vec3 total_emissive = emissive.rgb;

    outColor.rgb = total_emissive + calculate_lighting(cameraPosition, surface_color.rgb, pos, normal3, metallic_factor, roughness_factor, occlusion);
    outColor.rgb = reinhard_tone_mapping(outColor.rgb);
    outColor.rgb = srgb_from_rgb(outColor.rgb);
    outColor.a = surface_color.a;
}