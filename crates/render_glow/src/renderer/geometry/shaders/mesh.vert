
uniform mat4 viewProjection;
uniform mat4 modelMatrix;
in vec3 position;

// Rename these values to indicate that this is instance-level transforms here
in vec4 row1;
in vec4 row2;
in vec4 row3;

out vec3 pos;

#ifdef USE_NORMALS
uniform mat4 normalMatrix;
in vec3 normal;
out vec3 nor;
#endif

out vec4 col;

void main()
{
    // *** POSITION ***
    mat4 local2World = modelMatrix;

    #ifdef USE_INSTANCE_TRANSFORMS
    mat4 transform;
    transform[0] = vec4(row1.x, row2.x, row3.x, 0.0);
    transform[1] = vec4(row1.y, row2.y, row3.y, 0.0);
    transform[2] = vec4(row1.z, row2.z, row3.z, 0.0);
    transform[3] = vec4(row1.w, row2.w, row3.w, 1.0);
    local2World *= transform;
    #endif

    vec4 worldPosition = local2World * vec4(position, 1.);
    worldPosition /= worldPosition.w;
    gl_Position = viewProjection * worldPosition;

    pos = worldPosition.xyz;

    // *** NORMAL ***
    #ifdef USE_NORMALS
    #ifdef USE_INSTANCE_TRANSFORMS
    mat3 normalMat = mat3(transpose(inverse(local2World)));
    #else
    mat3 normalMat = mat3(normalMatrix);
    #endif
    nor = normalize(normalMat * normal);

    #endif

    // *** COLOR ***
    col = vec4(1.0);
}