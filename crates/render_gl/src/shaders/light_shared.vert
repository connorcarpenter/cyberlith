
vec3 calculate_light(vec3 light_color, vec3 light_dir, vec3 view_dir, vec3 normal, vec3 material_color, float material_shininess)
{
    // diffuse
    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    vec3 diffuse_color = diffuse_strength * light_color  * material_color;

    // specular
    vec3 reflect_dir = reflect(-light_dir, normal);
    float specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), material_shininess);
    // 0.5 here should be a material property
    vec3 specular_color = 0.5 * specular_strength * light_color;

    return diffuse_color + specular_color;
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
