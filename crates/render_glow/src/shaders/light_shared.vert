
vec3 fresnel_schlick_roughness(vec3 F0, float cosTheta, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(saturate(1.0 - cosTheta), 5.0);
}

// simple phong specular calculation with normalization
vec3 phong_specular(in vec3 view_dir, in vec3 light_dir, in vec3 normal, in vec3 specular_fresnel, in float roughness)
{
    vec3 reflectance = reflect(-light_dir, normal);
    float view_dot_reflectance = max(0.0, dot(view_dir, reflectance));

    float k = 1.999 / (roughness * roughness);

    return min(1.0, 3.0 * 0.0398 * k) * pow(view_dot_reflectance, min(10000.0, k)) * specular_fresnel;
}

vec3 calculate_light(vec3 light_color, vec3 light_dir, vec3 surface_color, vec3 view_dir, vec3 normal, float metallic, float roughness)
{
    // convert from right-handed y-up to left-handed z-up
    vec3 new_light_dir = vec3(light_dir.x, -light_dir.z, -light_dir.y);

    // compute material reflectance
    float normal_dot_light_dir = max(0.001, dot(normal, new_light_dir));
    float normal_dot_view_dir = max(0.001, dot(normal, view_dir));

    // mix between metal and non-metal material, for non-metal
    // constant base specular factor of 0.0 white is used
    vec3 F0 = mix(vec3(0.0), surface_color, metallic);

    // specular reflectance with PHONG
    vec3 specular_fresnel = fresnel_schlick_roughness(F0, normal_dot_view_dir, roughness);
    vec3 specular = phong_specular(view_dir, new_light_dir, normal, specular_fresnel, roughness);

    // diffuse is common for any model
    vec3 diffuse_fresnel = 1.0 - specular_fresnel;
    vec3 diffuse = diffuse_fresnel * mix(surface_color, vec3(0.0), metallic) / PI;
    
    // final result
    return (diffuse + specular) * light_color * normal_dot_light_dir;
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
