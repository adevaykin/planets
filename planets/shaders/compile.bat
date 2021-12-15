@echo Working directory: %cd%

@echo Compiling basic.vert
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\basic.vert -o %cd%\shaders\bin\basic.vert.spv
@echo Compiling basic.frag
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\basic.frag -o %cd%\shaders\bin\basic.frag.spv

@echo Compiling background.vert
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\background.vert -o %cd%\shaders\bin\background.vert.spv
@echo Compiling background.frag
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\background.frag -o %cd%\shaders\bin\background.frag.spv

@echo Compiling gameoflife.vert
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\gameoflife.vert -o %cd%\shaders\bin\gameoflife.vert.spv
@echo Compiling gameoflife.frag
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\gameoflife.frag -o %cd%\shaders\bin\gameoflife.frag.spv

@echo Done shaders compilation
