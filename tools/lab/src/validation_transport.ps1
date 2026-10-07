# Fixed PowerShell Direct cmdlet glue (DEC-020). Rust owns workload decisions.
__STAGING_PRELUDE__
try {
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'elevated development token required' }
    $request = [Console]::In.ReadToEnd() | ConvertFrom-Json -ErrorAction Stop
    $vm = & $getVmCommand -Id ([guid]$request.vm_id) -ErrorAction Stop
    $drives = @(& $getVmHardDiskDriveCommand -VM $vm -ErrorAction Stop)
    $adapters = @(Hyper-V\Get-VMGpuPartitionAdapter -VM $vm -ErrorAction Stop)
    if ($vm.Name -cne $request.vm_name -or $vm.State -ne 'Running' -or $vm.Generation -ne 2 -or
        $drives.Count -ne 1 -or $drives[0].Path -ine $request.child_path -or $adapters.Count -ne 1 -or
        $adapters[0].InstancePath -ine $request.gpu_interface -or @(& $getVmSnapshotCommand -VM $vm -ErrorAction Stop).Count -ne 0) { throw 'target identity mismatch' }
    $disk = & $getVhdCommand -Path $drives[0].Path -ErrorAction Stop
    $parent = Get-Item -LiteralPath $request.parent_path -Force -ErrorAction Stop
    $child = Get-Item -LiteralPath $request.child_path -Force -ErrorAction Stop
    if ($disk.VhdType -ne 'Differencing' -or $disk.ParentPath -ine $request.parent_path -or -not $parent.IsReadOnly -or
        ($parent.Attributes -band [IO.FileAttributes]::ReparsePoint) -or ($child.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'disk chain identity mismatch' }
    $secure = ConvertTo-SecureString ([string]$request.password) -AsPlainText -Force
    $credential = [pscredential]::new([string]$request.username, $secure)
    $request.password = $null
    $session = Open-FixedSession $request $credential
    try {
        $root = Invoke-Command -Session $session -ArgumentList @($request.root,$request.computer_name,$request.machine_guid,$aclValidatorSource) -ScriptBlock {
            param($Root,$Computer,$Guid,$AclSource)
            $ErrorActionPreference='Stop'
            if ($env:COMPUTERNAME -ine $Computer -or (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Cryptography').MachineGuid -cne $Guid) { throw 'guest identity mismatch' }
            $principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
            if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'guest administrator required' }
            & ([scriptblock]::Create($AclSource))
            $full=[IO.Path]::GetFullPath($Root).TrimEnd('\')
            $current=[IO.Path]::GetPathRoot($full)
            foreach ($segment in $full.Substring($current.Length).Split('\')) {
                $current=Join-Path $current $segment
                if (-not (Test-Path -LiteralPath $current)) { New-Item -ItemType Directory -Path $current | Out-Null }
                $item=Get-Item -LiteralPath $current -Force
                if (-not $item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'unsafe validation directory' }
            }
            Assert-ProtectedTree $full
            $lock=[IO.File]::Open((Join-Path $full 'validation.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
            $lock.Dispose()
            $full
        }
        foreach ($entry in $request.inputs) {
            $source=[string]$entry[0]
            $input=$entry[1]
            Assert-SafeRelative ([string]$input.name)
            if ([IO.Path]::GetFileName([string]$input.name) -cne [string]$input.name) { throw 'flat validation filenames required' }
            $item=Get-Item -LiteralPath $source -Force -ErrorAction Stop
            if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $input.bytes -or
                (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $input.sha256) { throw 'source identity mismatch' }
            if ([string]$input.name -like '*.dll') {
                $signature=Get-AuthenticodeSignature -LiteralPath $source -ErrorAction Stop
                if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notlike '*Microsoft Corporation*') { throw 'Microsoft runtime signature required' }
            }
            $target=Join-Path $root ([string]$input.name)
            Invoke-Command -Session $session -ArgumentList $target -ScriptBlock {
                param($Target)
                if (Test-Path -LiteralPath $Target) {
                    $item=Get-Item -LiteralPath $Target -Force
                    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'unsafe validation input' }
                    Assert-ProtectedFile $Target
                }
            }
            Copy-Item -LiteralPath $source -Destination $target -ToSession $session -Force -ErrorAction Stop
        }
        $result=Invoke-Command -Session $session -ArgumentList @($root,$request.worker_json,$request.timeout_ms) -ScriptBlock {
            param($Root,$InputJson,$Timeout)
            $ErrorActionPreference='Stop'
            $input=$InputJson | ConvertFrom-Json
            foreach ($file in $input.inputs) {
                $path=Join-Path $Root $file.name
                Assert-ProtectedTree $Root
                Assert-ProtectedFile $path
                $item=Get-Item -LiteralPath $path -Force
                if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $file.bytes -or
                    (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() -cne $file.sha256) { throw 'guest input integrity mismatch' }
            }
            $start=[Diagnostics.ProcessStartInfo]::new()
            $start.FileName=Join-Path $Root 'hyper-gpu-validation-worker.exe'
            $start.WorkingDirectory=$Root
            $start.UseShellExecute=$false
            $start.CreateNoWindow=$true
            $start.RedirectStandardInput=$true
            $start.RedirectStandardOutput=$true
            $start.RedirectStandardError=$true
            $start.StandardOutputEncoding=[Text.UTF8Encoding]::new($false)
            $start.StandardErrorEncoding=[Text.UTF8Encoding]::new($false)
            $process=[Diagnostics.Process]::Start($start)
            try {
                $stdout=$process.StandardOutput.ReadToEndAsync()
                $stderr=$process.StandardError.ReadToEndAsync()
                $process.StandardInput.Write($InputJson)
                $process.StandardInput.Close()
                if (-not $process.WaitForExit([int]$Timeout)) {
                    $process.Kill()
                    if (-not $process.WaitForExit(5000)) { throw 'worker cleanup timeout' }
                    throw 'worker deadline expired; remote process terminated'
                }
                $process.WaitForExit()
                if ($process.ExitCode -notin @(0,1) -or $stdout.Result.Length -gt 60000 -or $stderr.Result.Length -gt 4096) { throw 'invalid worker execution/output' }
                $stdout.Result
            } finally {
                if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit(5000) | Out-Null }
                $process.Dispose()
            }
        }
        [Console]::Out.Write([string]$result)
    } finally { Remove-PSSession -Session $session -ErrorAction Stop }
} catch {
    Emit 'error' 'validation-transport-failed'
    exit 1
}
