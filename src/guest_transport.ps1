# Fixed PowerShell Direct interface glue; all driver/file decisions belong to Rust.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$session = $null
try {
    Import-Module '__HYPERV_MODULE__' -ErrorAction Stop
    $r = [Console]::In.ReadToEnd() | ConvertFrom-Json
    $secure = ConvertTo-SecureString ([string]$r.password) -AsPlainText -Force
    $credential = [pscredential]::new([string]$r.username, $secure)
    $r.password = $null
    # Running is not session readiness. Retry connection only, before any mutation.
    $connectDeadline = [DateTime]::UtcNow.AddMinutes(3)
    while (-not $session) {
        try { $session = New-PSSession -VMId ([guid]$r.vm_id) -Credential $credential -ErrorAction Stop }
        catch {
            if ($_.Exception -is [UnauthorizedAccessException] -or $_.CategoryInfo.Category -eq 'AuthenticationError' -or [DateTime]::UtcNow -ge $connectDeadline) { throw }
            Start-Sleep -Seconds 2
        }
    }
    $root = Invoke-Command -Session $session -ScriptBlock {
        $path = Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'HyperGpuSupport\Guest'
        foreach ($p in @((Split-Path $path), $path)) {
            $existed = Test-Path -LiteralPath $p
            if (-not $existed) { [void][IO.Directory]::CreateDirectory($p) }
            $current = $p
            while ($current) {
                if ((Get-Item -LiteralPath $current -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe bootstrap path' }
                $current = Split-Path $current
            }
            $acl = [Security.AccessControl.DirectorySecurity]::new()
            $acl.SetAccessRuleProtection($true, $false)
            $admin = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-544')
            if ($existed) {
                $owner = (Get-Acl -LiteralPath $p).GetOwner([Security.Principal.SecurityIdentifier]).Value
                if ($owner -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted bootstrap owner' }
            }
            $acl.SetOwner($admin)
            foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
                $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid), 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'))
            }
            Set-Acl -LiteralPath $p -AclObject $acl
        }
        $path
    }
    # Validate existing leaf objects before removing only the link/name, then copy
    # into the protected parent. Never overwrite a pre-existing reparse/hard link.
    $destinations = @()
    foreach ($a in $r.artifacts) { $destinations += Join-Path $root $a.name }
    Invoke-Command -Session $session -ArgumentList (,$destinations) -ScriptBlock {
        param($paths)
        foreach ($path in $paths) {
            if (Test-Path -LiteralPath $path) {
                $item = Get-Item -LiteralPath $path -Force
                if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe bootstrap leaf' }
                $owner = (Get-Acl -LiteralPath $path).GetOwner([Security.Principal.SecurityIdentifier]).Value
                if ($owner -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted bootstrap leaf owner' }
                [IO.File]::Delete($path)
            }
        }
    }
    foreach ($a in $r.artifacts) {
        Copy-Item -LiteralPath (Join-Path $r.install $a.name) -Destination (Join-Path $root $a.name) -ToSession $session -Force
        Invoke-Command -Session $session -ArgumentList (Join-Path $root $a.name) -ScriptBlock {
            param($path)
            $acl = Get-Acl -LiteralPath $path
            $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
            Set-Acl -LiteralPath $path -AclObject $acl
        }
    }
    $bundle = Join-Path $root $r.digest
    Invoke-Command -Session $session -ArgumentList $bundle -ScriptBlock {
        param($path)
        if (Test-Path -LiteralPath $path) {
            if ((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe bundle' }
            $owner = (Get-Acl -LiteralPath $path).GetOwner([Security.Principal.SecurityIdentifier]).Value
            if ($owner -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted bundle owner' }
        } else { [void][IO.Directory]::CreateDirectory($path) }
        $acl = Get-Acl -LiteralPath (Split-Path $path)
        Set-Acl -LiteralPath $path -AclObject $acl
    }
    if ($r.mode -eq 'prepare') {
        $destinations = @((Join-Path $bundle 'manifest.json'))
        for ($i = 0; $i -lt $r.manifest.files.Count; $i++) { $destinations += Join-Path $bundle ([string]$i) }
        Invoke-Command -Session $session -ArgumentList (,$destinations) -ScriptBlock {
            param($paths)
            foreach ($path in $paths) {
                if (Test-Path -LiteralPath $path) {
                    $item = Get-Item -LiteralPath $path -Force
                    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe transfer leaf' }
                    $owner = (Get-Acl -LiteralPath $path).GetOwner([Security.Principal.SecurityIdentifier]).Value
                    if ($owner -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted transfer leaf owner' }
                    [IO.File]::Delete($path)
                }
            }
        }
        for ($i = 0; $i -lt $r.manifest.files.Count; $i++) {
            Copy-Item -LiteralPath $r.manifest.files[$i].source -Destination (Join-Path $bundle ([string]$i)) -ToSession $session -Force
        }
        $json = $r.manifest | ConvertTo-Json -Depth 12 -Compress
        Invoke-Command -Session $session -ArgumentList $bundle, $json -ScriptBlock {
            param($path, $json)
            [IO.File]::WriteAllText((Join-Path $path 'manifest.json'), $json, [Text.UTF8Encoding]::new($false))
            foreach ($file in Get-ChildItem -LiteralPath $path -File) {
                $acl = Get-Acl -LiteralPath $file.FullName
                $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
                Set-Acl -LiteralPath $file.FullName -AclObject $acl
            }
        }
    }
    $output = Invoke-Command -Session $session -ArgumentList $root, $bundle, $r.mode, $r.vendor, $r.device, $r.artifacts -ScriptBlock {
        param($root, $bundle, $mode, $vendor, $device, $artifacts)
        $ErrorActionPreference = 'Stop'
        $handles = @()
        try {
            foreach ($a in $artifacts) {
                $path = Join-Path $root $a.name
                if ((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe worker' }
                $handles += [IO.File]::Open($path, 'Open', 'Read', 'Read')
                if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $a.hash) { throw 'Worker integrity failure' }
            }
            $result = & (Join-Path $root 'hyper-gpu-guest.exe') $mode $bundle ([string]$vendor) ([string]$device)
            if ($LASTEXITCODE -ne 0) { throw 'Guest worker failed' }
            $result -join "`n"
        } finally { foreach ($handle in $handles) { $handle.Dispose() } }
    }
    @{ output = [string]$output } | ConvertTo-Json -Compress
} catch {
    [Console]::Error.WriteLine('Fixed guest transport failed; reconcile before retry.')
    exit 1
} finally { if ($session) { Remove-PSSession -Session $session } }
