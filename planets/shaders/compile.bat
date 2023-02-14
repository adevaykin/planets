@echo Working directory: %cd%

@rd /S /Q bin
@mkdir bin

@for %%A in (*.vert *.frag) do (
	call glslc %%A -o "bin/%%A.spv"
)

@echo Done shaders compilation
