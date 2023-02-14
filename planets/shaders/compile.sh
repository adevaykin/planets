#!/bin/bash

rm -rf bin;
mkdir -p bin;

for file in *.vert *.frag;
do
  echo "Compiling $file"
  glslc "$file" -o "bin/$file.spv"
done
