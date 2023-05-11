#ifndef _ALBEDO_LIGHTMAPPER_H
#define _ALBEDO_LIGHTMAPPER_H

#ifdef __cplusplus
namespace Albedo {

extern "C" {
#endif

struct AttributeSlice {
    const unsigned char* data;
    unsigned int stride;
};

struct MeshDescriptor {
    AttributeSlice positions;
    AttributeSlice normals;
    AttributeSlice uvs;
    const unsigned int* indices;
    unsigned int vertex_count;
    unsigned int index_count;
};

struct ImageSlice {
    unsigned int width;
    unsigned int height;
    unsigned char* data;
};

void init();
void set_mesh_data(MeshDescriptor descriptor);
void bake(ImageSlice slice);

#ifdef __cplusplus
}}

#endif
#endif