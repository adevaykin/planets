#!/bin/bash

export PATH=$PATH:~/VulkanSDK/latest/macOS/bin

rm -rf bin;
mkdir -p bin;

for file in *.vert *.frag;
do
  echo "Compiling $file"
  glslc "$file" -o "bin/$file.spv"
done
