# Privilege: elevated. Explicit disposable-test fixture only, never product logic.
#Requires -RunAsAdministrator
$ErrorActionPreference = 'Stop'
$r = [Console]::In.ReadToEnd() | ConvertFrom-Json
Import-Module Hyper-V
$vm = Get-VM -Id ([guid]$r.vm_id)
$disks = @(Get-VMHardDiskDrive -VM $vm)
if ($vm.Generation -ne 2 -or $disks.Count -ne 1 -or $disks[0].Path -ine $r.child -or (Get-VHD -Path $r.child).ParentPath -ine $r.parent) { throw 'Disposable VM/disk mismatch' }
if (@(Get-VMHostPartitionableGpu | Where-Object Name -ceq $r.gpu).Count -ne 1) { throw 'Disposable GPU mismatch' }
if ($r.mode -eq 'snapshot') {
    @{vm_id=$vm.Id.ToString();generation=$vm.Generation;state=[string]$vm.State;disk=$disks[0].Path;parent=$r.parent;processors=(Get-VMProcessor -VM $vm).Count;memory=$vm.MemoryStartup;dynamic_memory=$vm.DynamicMemoryEnabled;secure_boot=[string](Get-VMFirmware -VM $vm).SecureBoot;tpm=(Get-VMSecurity -VM $vm).TpmEnabled;gpu=@(Get-VMGpuPartitionAdapter -VM $vm | Select-Object InstancePath);host_build=[Environment]::OSVersion.Version.ToString();architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()} | ConvertTo-Json -Depth 6 -Compress
    exit 0
}
if ($r.mode -notin @('stage','readback') -or $vm.State -ne 'Running') { throw 'Invalid fixture state/mode' }
if ($r.mode -eq 'stage' -and @(Get-VMGpuPartitionAdapter -VM $vm).Count) { throw 'Stage requires detached GPU' }
$secure = ConvertTo-SecureString ([string]$r.credential.password) -AsPlainText -Force
$credential = [pscredential]::new([string]$r.credential.username, $secure)
$r.credential = $null
$session = $null
try {
    $deadline = [DateTime]::UtcNow.AddMinutes(3)
    while (!$session) {
        try { $session = New-PSSession -VMId $vm.Id -Credential $credential }
        catch { if ([DateTime]::UtcNow -ge $deadline -or $_.CategoryInfo.Category -eq 'AuthenticationError') { throw }; Start-Sleep -Seconds 2 }
    }
    $result = Invoke-Command -Session $session -ArgumentList $r.mode,$r.destination,$r.digest -ScriptBlock {
        param($mode,$relative,$digest)
        $ErrorActionPreference = 'Stop'
        if ($digest -notmatch '^[a-f0-9]{64}$' -or $relative -notlike 'System32\HostDriverStore\*.dll' -or $relative.Contains('..')) { throw 'Unsafe dynamic destination' }
        $path = [IO.Path]::GetFullPath((Join-Path $env:windir $relative))
        $expectedRoot = (Join-Path $env:windir 'System32\HostDriverStore') + '\'
        if (!$path.StartsWith($expectedRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Destination escaped managed root' }
        $ancestor = $path
        while ($ancestor) { if ((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse fixture path' }; $ancestor = Split-Path $ancestor }
        $bundle = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) "HyperGpuSupport\Guest\$digest"
        $manifest = Get-Content -LiteralPath (Join-Path $bundle 'manifest.json') -Raw | ConvertFrom-Json
        $file = @($manifest.files | Where-Object destination -ceq $relative)
        if ($file.Count -ne 1) { throw 'Dynamic destination is not in previous managed payload' }
        $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -cne $file[0].sha256) { throw 'Fixture preimage/readback hash mismatch' }
        if ($mode -eq 'stage') {
            $backup = Join-Path $bundle 'qualification-preimage.dll'
            if (Test-Path -LiteralPath $backup) { throw 'Existing fixture preimage requires inspection' }
            Move-Item -LiteralPath $path -Destination $backup
            @{destination=$relative;sha256=$hash;bytes=$file[0].bytes;preimage=$backup;removed_for_fresh_write=$true}
        } else {
            $receipt = Get-Content -LiteralPath (Join-Path (Split-Path $bundle) 'prepared.json') -Raw | ConvertFrom-Json
            if ($receipt.digest -cne $digest) { throw 'Fresh preparation receipt mismatch' }
            $backup = Join-Path $bundle 'qualification-preimage.dll'
            if ((Get-Item -LiteralPath $backup -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe preimage' }
            if ((Get-FileHash -LiteralPath $backup -Algorithm SHA256).Hash.ToLowerInvariant() -cne $hash) { throw 'Preimage does not match independently prepared destination' }
            Remove-Item -LiteralPath $backup
            $os = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
            @{destination=$relative;sha256=$hash;bytes=$file[0].bytes;receipt=$receipt;guest_build="$($os.CurrentBuildNumber).$($os.UBR)";architecture=[Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString();preimage_retired=$true}
        }
    }
    $result | ConvertTo-Json -Depth 7 -Compress
} catch { [Console]::Error.WriteLine('Disposable fixture failed; retain and inspect preimages.'); exit 1 }
finally { if ($session) { Remove-PSSession -Session $session } }
