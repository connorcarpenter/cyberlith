
struct BaseLight
{
    vec3 color;
    float intensity;
};

vec3 fresnel_schlick_roughness(vec3 F0, float cosTheta, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(saturate(1.0 - cosTheta), 5.0);
}

// simple phong specular calculation with normalization
vec3 phong_specular(in vec3 V, in vec3 L, in vec3 N, in vec3 specular_fresnel, in float roughness)
{
    vec3 R = reflect(-L, N);
    float VdR = max(0.0, dot(V, R));

    float k = 1.999 / (roughness * roughness);

    return min(1.0, 3.0 * 0.0398 * k) * pow(VdR, min(10000.0, k)) * specular_fresnel;
}

// L - light_direction
// V - view direction
// N - normal
vec3 calculate_light(vec3 light_color, vec3 L, vec3 surface_color, vec3 V, vec3 N, float metallic, float roughness)
{
    // compute material reflectance
    float NdL = max(0.001, dot(N, L));
    float NdV = max(0.001, dot(N, V));

    // mix between metal and non-metal material, for non-metal
    // constant base specular factor of 0.04 grey is used
    vec3 F0 = mix(vec3(0.04), surface_color, metallic);

    // specular reflectance with PHONG
    vec3 specular_fresnel = fresnel_schlick_roughness(F0, NdV, roughness);
    vec3 specular = phong_specular(V, L, N, specular_fresnel, roughness);

    // diffuse is common for any model
    vec3 diffuse_fresnel = 1.0 - specular_fresnel;
    vec3 diffuse = diffuse_fresnel * mix(surface_color, vec3(0.0), metallic) / PI;
    
    // final result
    return (diffuse + specular) * light_color * NdL;
}

vec3 attenuate(vec3 light_color, vec3 attenuation, float distance)
{
    float att =  attenuation.x +
        attenuation.y * distance +
        attenuation.z * distance * distance;

    return light_color / max(1.0, att);
}

float is_visible(sampler2D shadowMap, vec4 shadow_coord, vec2 offset)
{
    vec2 uv = (shadow_coord.xy + offset)/shadow_coord.w;
    if(uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0;
    }
    float shadow_cast_distance = texture(shadowMap, uv).x;
    if(shadow_cast_distance > 0.999) {
        return 1.0;
    }
    float true_distance = (shadow_coord.z - 0.005)/shadow_coord.w;
    return shadow_cast_distance > true_distance ? 1.0 : 0.0;
}

float calculate_shadow(sampler2D shadowMap, mat4 shadowMVP, vec3 position)
{
    vec4 shadow_coord = shadowMVP * vec4(position, 1.);
    float visibility = 0.0;
    vec2 poissonDisk[4] = vec2[](
                                 vec2( -0.94201624, -0.39906216 ),
                                 vec2( 0.94558609, -0.76890725 ),
                                 vec2( -0.094184101, -0.92938870 ),
                                 vec2( 0.34495938, 0.29387760 )
                                 );
    for (int i=0;i<4;i++)
    {
        visibility += is_visible(shadowMap, shadow_coord, poissonDisk[i] * 0.001f);
    }
    return visibility * 0.25;
}
