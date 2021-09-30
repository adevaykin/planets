%cd%

%VULKAN_SDK%\Bin32\glslc.exe shaders\basic.vert -o shaders\bin\basic.vert.spv
%VULKAN_SDK%\Bin32\glslc.exe shaders\basic.frag -o shaders\bin\basic.frag.spv

%VULKAN_SDK%\Bin32\glslc.exe shaders\background.vert -o shaders\bin\background.vert.spv
%VULKAN_SDK%\Bin32\glslc.exe shaders\background.frag -o shaders\bin\background.frag.spv