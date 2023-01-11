@echo Working directory: %cd%

@mkdir bin

@for %%A in (*.vert *.frag) do (
	call glslc %%A -o "bin/%%A.spv"
)

@echo Done shaders compilation
