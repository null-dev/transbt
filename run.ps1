# Use PSExec64 from Sysinternals PsTools to elevate to SYSTEM user
$ProcessPath = "C:\Users\Andy\Desktop\Software\PSTools\PsExec64.exe"

# Prepend arguments required by PSExec64
$FullArgs = @("-s", "-i", "$($PWD.Path)\_run.bat") + $args

# Start the process with the given arguments
try {
    # -verb RunAs makes the new process perform UAC elevation
    Start-Process -FilePath $ProcessPath -ArgumentList $FullArgs -Wait -verb RunAs
} catch {
    Write-Host "Error: Failed to start the process."
    Write-Host $_.Exception.Message
}