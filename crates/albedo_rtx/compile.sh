#!/bin/bash

to_spirv()
{
  $TOOLS_DIR/vulkansdk-macos-1.2.135.0/macOS/bin/glslangValidator -V $1 -o $2
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

