@echo off
REM Double-click this file to open a terminal with ven configured
powershell -NoExit -ExecutionPolicy Bypass -Command "$env:VEN = 'd:\projects\software\ven\target\debug\ven.exe'; Write-Host '=== ven Test Terminal ===' -ForegroundColor Cyan; Write-Host 'VEN: ' -NoNewline; Write-Host $env:VEN -ForegroundColor Green; Write-Host 'Hook: ven shell hook powershell' -ForegroundColor Yellow; Write-Host 'Ready to test!' -ForegroundColor Green; function v { & $env:VEN @args }; Set-Location d:\projects\software\ven\example"
