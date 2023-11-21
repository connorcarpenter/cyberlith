
in vec3 nor;

layout (location = 0) out vec4 outColor;

void main()
{

    vec3 normal = normalize(gl_FrontFacing ? nor : -nor);
    outColor = vec4(0.5 + 0.5 * normal, 1.0);
}