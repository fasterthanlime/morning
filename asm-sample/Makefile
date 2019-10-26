
MSVC := /d/Programs/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC/14.23.28105/bin/HostX64/x64

.PHONY: all

all:
	nasm -f win64 -gcv8 hello.asm
	"${MSVC}/link.exe" //SUBSYSTEM:CONSOLE //ENTRY:_start hello.obj
	./hello
