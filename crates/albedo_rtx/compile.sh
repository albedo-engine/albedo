#!/bin/bash

COMPILER=/d/dev/shaderc/bin/glslc

to_spirv()
{
    file=$1
    filename="$(basename -- $file)"
    output_path=./src/shaders/spirv/$filename.spv
    echo "Compiling into $output_path..."
    $COMPILER $file -o $output_path
    echo "Compilation done!"
    echo ""
}

for file in ./src/shaders/*.comp; do
    to_spirv $file
done
for file in ./src/shaders/*.frag; do
    to_spirv $file
done
for file in ./src/shaders/*.vert; do
    to_spirv $file
done
