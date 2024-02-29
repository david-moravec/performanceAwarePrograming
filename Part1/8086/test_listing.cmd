@echo off
nasm.exe -f bin %1 -o src\assembled

cd src

clang.exe -o 8086.exe main.c Disassemble.c RegEnum.c

8086.exe assembled > output
nasm.exe -f bin output -o my_assembled

fc assembled my_assembled
del output
del assembled
del my_assembled

cd ..

