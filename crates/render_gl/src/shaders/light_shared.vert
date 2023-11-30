
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
