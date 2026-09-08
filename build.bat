@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
%USERPROFILE%\.cargo\bin\cargo.exe build --release