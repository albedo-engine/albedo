#!/bin/bash

COMPILER=~/Tools/shaderc/bin/glslc

to_spirv()
{
    $COMPILER $1 -o $2
}


for file in ./src/shaders/*.comp
do
    if [[ -f $file ]]; then
        filename="$(basename -- $file)"
        output_path=./src/shaders/spirv/$filename.spv
        echo "Compiling into $output_path..."
        to_spirv $file $output_path
        echo "Compilation done!"
        echo ""
    fi
done

