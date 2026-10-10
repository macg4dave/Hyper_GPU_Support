# Fixed PowerShell Direct interface glue; all driver/file decisions belong to Rust.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$session = $null
$diagnostic = $null
function Write-Stage([string]$stage, [string]$status, [string]$context, [long]$duration = 0) {
    if (!$diagnostic) { return }
    $line = @{utc=[DateTime]::UtcNow.ToString('o');pid=$PID;vm_id=[string]$r.vm_id;mode=[string]$r.mode;stage=$stage;status=$status;context=$context;duration_ms=$duration;significant_delay=($duration -ge 5000)} | ConvertTo-Json -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($line + "`n")
    $diagnostic.Write($bytes, 0, $bytes.Length)
    $diagnostic.Flush($true)
}
function Copy-Logged([string]$source, [string]$destination) {
    Write-Stage 'file-copy' 'start' "$source -> $destination"
    $timer = [Diagnostics.Stopwatch]::StartNew()
    try {
        Copy-Item -LiteralPath $source -Destination $destination -ToSession $session -Force
        Write-Stage 'file-copy' 'done' "$source -> $destination" $timer.ElapsedMilliseconds
    } catch {
        Write-Stage 'file-copy' 'failed' "$source -> $destination; HRESULT=$($_.Exception.HResult)" $timer.ElapsedMilliseconds
        throw
    }
}
try {
    Import-Module '__HYPERV_MODULE__' -ErrorAction Stop
    $r = [Console]::In.ReadToEnd() | ConvertFrom-Json
    if ($r.diagnostics) {
        $path = Join-Path $r.install ("diagnostics/bridge-" + [DateTime]::UtcNow.Ticks + "-$PID.jsonl")
        $diagnostic = [IO.FileStream]::new($path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    }
    $timer = [Diagnostics.Stopwatch]::StartNew()
    Write-Stage 'session-connect' 'start' 'PowerShell Direct'
    $secure = ConvertTo-SecureString ([string]$r.password) -AsPlainText -Force
    $credential = [pscredential]::new([string]$r.username, $secure)
    $r.password = $null
    # Running is not session readiness. Retry connection only, before any mutation.
    $connectDeadline = [DateTime]::UtcNow.AddMinutes(3)
    while (-not $session) {
        try { $session = New-PSSession -VMId ([guid]$r.vm_id) -Credential $credential -ErrorAction Stop }
        catch {
            if ($_.Exception -is [UnauthorizedAccessException] -or $_.CategoryInfo.Category -eq 'AuthenticationError' -or [DateTime]::UtcNow -ge $connectDeadline) { throw }
            Write-Stage 'session-connect' 'delay' "connection not ready; HRESULT=$($_.Exception.HResult)" $timer.ElapsedMilliseconds
            Start-Sleep -Seconds 2
        }
    }
    Write-Stage 'session-connect' 'done' 'PowerShell Direct' $timer.ElapsedMilliseconds
    $timer.Restart()
    Write-Stage 'bootstrap-path' 'start' 'protect guest root'
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
    Write-Stage 'bootstrap-path' 'done' $root $timer.ElapsedMilliseconds
    $timer.Restart()
    Write-Stage 'bootstrap-leaves' 'start' $root
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
    Write-Stage 'bootstrap-leaves' 'done' $root $timer.ElapsedMilliseconds
    foreach ($a in $r.artifacts) {
        Copy-Logged (Join-Path $r.install $a.name) (Join-Path $root $a.name)
        $timer.Restart()
        Write-Stage 'bootstrap-acl' 'start' $a.name
        Invoke-Command -Session $session -ArgumentList (Join-Path $root $a.name) -ScriptBlock {
            param($path)
            $acl = Get-Acl -LiteralPath $path
            $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
            Set-Acl -LiteralPath $path -AclObject $acl
        }
        Write-Stage 'bootstrap-acl' 'done' $a.name $timer.ElapsedMilliseconds
    }
    $timer.Restart()
    Write-Stage 'bundle-path' 'start' $root
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
    Write-Stage 'bundle-path' 'done' $bundle $timer.ElapsedMilliseconds
    if ($r.mode -eq 'prepare') {
        $timer.Restart()
        Write-Stage 'transfer-leaves' 'start' $bundle
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
        Write-Stage 'transfer-leaves' 'done' $bundle $timer.ElapsedMilliseconds
        for ($i = 0; $i -lt $r.manifest.files.Count; $i++) {
            Copy-Logged $r.manifest.files[$i].source (Join-Path $bundle ([string]$i))
        }
        $timer.Restart()
        Write-Stage 'manifest-copy-and-acls' 'start' $bundle
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
    Write-Stage 'transfer-complete' 'done' $bundle $timer.ElapsedMilliseconds
    $timer.Restart()
    Write-Stage 'guest-integrity-and-worker' 'start' $r.mode
    $output = Invoke-Command -Session $session -ArgumentList $root, $bundle, $r.mode, $r.vendor, $r.device, $r.artifacts, $r.diagnostics -ScriptBlock {
        param($root, $bundle, $mode, $vendor, $device, $artifacts, $diagnostics)
        $ErrorActionPreference = 'Stop'
        $handles = @()
        $marker = Join-Path $root 'diagnostics.enabled'
        $ownedMarker = $false
        if ($diagnostics) {
            $directory = Join-Path $root 'diagnostics'
            if (Test-Path -LiteralPath $directory) {
                if ((Get-Item -LiteralPath $directory).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unsafe guest diagnostic directory' }
                if ((Get-Acl -LiteralPath $directory).GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted guest diagnostic directory' }
            } else { [void][IO.Directory]::CreateDirectory($directory) }
            Set-Acl -LiteralPath $directory -AclObject (Get-Acl -LiteralPath $root)
            if (!(Test-Path -LiteralPath $marker)) {
                $file = [IO.File]::Open($marker, 'CreateNew', 'Write', 'Read')
                $file.Flush($true)
                $file.Dispose()
                Set-Acl -LiteralPath $marker -AclObject (Get-Acl -LiteralPath $root)
                $ownedMarker = $true
            }
        }
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
        } finally { foreach ($handle in $handles) { $handle.Dispose() }; if ($ownedMarker) { [IO.File]::Delete($marker) } }
    }
    Write-Stage 'guest-integrity-and-worker' 'done' $r.mode $timer.ElapsedMilliseconds
    @{ output = [string]$output } | ConvertTo-Json -Compress
} catch {
    Write-Stage 'transport' 'failed' "HRESULT=$($_.Exception.HResult); type=$($_.Exception.GetType().FullName)"
    [Console]::Error.WriteLine('Fixed guest transport failed; reconcile before retry.')
    exit 1
} finally {
    if ($session) {
        Write-Stage 'session-remove' 'start' 'PowerShell Direct'
        Remove-PSSession -Session $session
        Write-Stage 'session-remove' 'done' 'PowerShell Direct'
    }
    if ($diagnostic) { $diagnostic.Dispose() }
}
