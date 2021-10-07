@echo Working directory: %cd%

@echo Compiling basic.vert
@%VULKAN_SDK%\Bin32\glslc.exe %cd%\shaders\basic.vert -o %cd%\shaders\bin\basic.vert.spv
@echo Compiling basic.frag
@%VULKAN_SDK%\Bin32\glslc.exe %cd%\shaders\basic.frag -o %cd%\shaders\bin\basic.frag.spv

@echo Compiling background.vert
@%VULKAN_SDK%\Bin32\glslc.exe %cd%\shaders\background.vert -o %cd%\shaders\bin\background.vert.spv
@echo Compiling background.frag
@%VULKAN_SDK%\Bin32\glslc.exe %cd%\shaders\background.frag -o %cd%\shaders\bin\background.frag.spv

@echo Done shaders compilation