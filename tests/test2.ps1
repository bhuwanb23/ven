function global:Set-Location {
    Microsoft.PowerShell.Management\Set-Location @args
    echo "cd hooked!"
}
