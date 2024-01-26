
vec3 calculate_light(vec3 light_color, vec3 light_dir, vec3 view_dir, vec3 normal, vec3 material_color, vec2 material_shine)
{
    // diffuse
    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    vec3 diffuse_color = diffuse_strength * light_color  * material_color;

    // specular
    vec3 reflect_dir = reflect(-light_dir, normal);
    float shine_size = material_shine.x;
    float shine_amount = material_shine.y;
    float specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), shine_size);
    vec3 specular_color = shine_amount * specular_strength * light_color;

    return diffuse_color + specular_color;
}

vec3 attenuate(vec3 light_color, vec3 attenuation, float distance)
{
    float att =  attenuation.x +
        attenuation.y * distance +
        attenuation.z * distance * distance;

    return light_color / max(1.0, att);
}
