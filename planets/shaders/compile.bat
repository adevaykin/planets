@echo Working directory: %cd%

@echo Compiling scenemodels.vert
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\scenemodels.vert -o %cd%\shaders\bin\scenemodels.vert.spv
@echo Compiling scenemodels.frag
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\scenemodels.frag -o %cd%\shaders\bin\scenemodels.frag.spv

@echo Compiling background.vert
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\background.vert -o %cd%\shaders\bin\background.vert.spv
@echo Compiling background.frag
@%VULKAN_SDK%\Bin\glslc.exe %cd%\shaders\background.frag -o %cd%\shaders\bin\background.frag.spv

@echo Done shaders compilation
