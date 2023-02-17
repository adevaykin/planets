#!/bin/bash

export PATH=$PATH:~/VulkanSDK/latest/macOS/bin

mkdir -p bin;

for file in *.vert *.frag;
do
  echo "Compiling $file"
  glslc "$file" -o "bin/$file.spv"
done
