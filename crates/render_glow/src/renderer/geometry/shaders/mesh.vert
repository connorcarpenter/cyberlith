
uniform mat4 viewProjection;
uniform mat4 modelMatrix;
in vec3 position;

in vec4 transform_row1;
in vec4 transform_row2;
in vec4 transform_row3;

out vec3 pos;

uniform mat4 normalMatrix;
flat out mat3 normalMat;

void main()
{
    // *** POSITION ***
    mat4 local2World = modelMatrix;

    #ifdef USE_INSTANCE_TRANSFORMS
    mat4 transform;
    transform[0] = vec4(transform_row1.x, transform_row2.x, transform_row3.x, 0.0);
    transform[1] = vec4(transform_row1.y, transform_row2.y, transform_row3.y, 0.0);
    transform[2] = vec4(transform_row1.z, transform_row2.z, transform_row3.z, 0.0);
    transform[3] = vec4(transform_row1.w, transform_row2.w, transform_row3.w, 1.0);
    local2World *= transform;
    #endif

    vec4 worldPosition = local2World * vec4(position, 1.);
    worldPosition /= worldPosition.w;
    gl_Position = viewProjection * worldPosition;

    pos = worldPosition.xyz;

    // *** NORMAL ***
    #ifdef USE_INSTANCE_TRANSFORMS
    normalMat = mat3(transpose(inverse(local2World)));
    #else
    normalMat = mat3(normalMatrix);
    #endif
}