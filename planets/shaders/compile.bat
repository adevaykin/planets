@echo Working directory: %cd%

@rd /S /Q bin
@mkdir bin

@for %%A in (*.vert *.frag *.rchit *.rmiss *.rgen) do (
	call glslc %%A -O --target-env=vulkan1.2 -o "bin/%%A.spv"
)

@echo Done shaders compilation
