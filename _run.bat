@echo off
:: CD to directory to where script is located
cd "%~dp0"
target\release\transbt.exe %*
pause