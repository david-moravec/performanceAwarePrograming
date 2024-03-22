@echo off
nasm.exe -f bin %1 -o rust_decode\assembled

cd rust_decode


cargo run assembled > output
nasm.exe -f bin output -o my_assembled

fc assembled my_assembled
del output
del assembled
del my_assembled

cd ..

