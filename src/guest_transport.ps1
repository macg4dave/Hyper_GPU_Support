# Fixed PowerShell Direct interface glue; all driver/file decisions belong to Rust.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$session = $null
$diagnostic = $null
$checkingCompatibility = $false
# Transport this fixed read-only query into the actual preparation session.
$readInstallationIdentity = {
    $os = @(Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop)
    if ($os.Count -ne 1) { throw 'Ambiguous OS identity' }
    $version = [version]$os[0].Version
    $build = [uint32]$os[0].BuildNumber
    $productType = [uint32]$os[0].ProductType
    $kind = Get-ItemPropertyValue -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion' -Name InstallationType -ErrorAction Stop
    if ($null -eq $version -or $version.Build -lt 0 -or $version.Revision -ge 0 -or $version.Build -ne $build -or $productType -notin @(1,2,3) -or [string]::IsNullOrEmpty($kind)) {
        throw 'Incomplete or inconsistent OS identity'
    }
    @{ major = $version.Major; minor = $version.Minor; build = $build; workstation = ($productType -eq 1 -and $kind -ceq 'Client') } | ConvertTo-Json -Compress
}
function Write-Stage([string]$stage, [string]$status, [string]$context, [long]$duration = 0) {
    if (!$diagnostic) { return }
    $line = @{utc=[DateTime]::UtcNow.ToString('o');pid=$PID;vm_id=[string]$r.vm_id;mode=[string]$r.mode;stage=$stage;status=$status;context=$context;duration_ms=$duration;significant_delay=($duration -ge 5000)} | ConvertTo-Json -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($line + "`n")
    $diagnostic.Write($bytes, 0, $bytes.Length)
    $diagnostic.Flush($true)
}
$transferIndex = 0
$transferTotal = [long]0
function Write-Transfer([string]$kind, [long]$bytes) {
    [Console]::Error.WriteLine("HGTRANSFER $kind $script:transferIndex $bytes")
}
function Copy-Logged([string]$source, [string]$destination, [long]$expectedBytes) {
    Write-Stage 'file-copy' 'start' "$source -> $destination"
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $inputFile = $null
    Write-Transfer 'begin' 0
    try {
        # Fixed stream transfer only: Rust owns membership, lengths, deadlines,
        # acknowledgement validation and final independent guest authentication.
        $inputFile = [IO.File]::Open($source, 'Open', 'Read', 'Read')
        if ($inputFile.Length -ne $expectedBytes) { throw 'Transfer source length changed' }
        Invoke-Command -Session $session -ArgumentList $destination -ScriptBlock {
            param($path)
            $global:hyperGpuTransfer = [IO.File]::Open($path, 'Create', 'Write', 'None')
        }
        $buffer = [byte[]]::new(__TRANSFER_CHUNK_BYTES__)
        $copied = [long]0
        while (($count = $inputFile.Read($buffer, 0, $buffer.Length)) -gt 0) {
            if ($count -gt ($expectedBytes - $copied)) { throw 'Transfer source grew' }
            # Send only bytes read; a small file or final fragment must not
            # serialize the unused portion of the fixed-size buffer.
            $chunk = $buffer
            if ($count -lt $buffer.Length) {
                $chunk = [byte[]]::new($count)
                [Buffer]::BlockCopy($buffer, 0, $chunk, 0, $count)
            }
            # A byte-array is one typed remoting argument, never a command/path.
            $ack = Invoke-Command -Session $session -ArgumentList $chunk, $count -ScriptBlock {
                param([byte[]]$chunk, [int]$count)
                if ($count -le 0 -or $count -ne $chunk.Length) { throw 'Invalid transfer chunk length' }
                $global:hyperGpuTransfer.Write($chunk, 0, $count)
                $global:hyperGpuTransfer.Flush()
                $global:hyperGpuTransfer.Position
            }
            $chunk = $null
            $copied += $count
            if ([long]$ack -ne $copied) { throw 'Transfer acknowledgement mismatch' }
            Write-Transfer 'bytes' $copied
        }
        if ($copied -ne $expectedBytes) { throw 'Transfer source shortened' }
        Invoke-Command -Session $session -ScriptBlock {
            $global:hyperGpuTransfer.Flush($true)
            $global:hyperGpuTransfer.Dispose()
            $global:hyperGpuTransfer = $null
        }
        Write-Transfer 'end' $copied
        $script:transferTotal += $copied
        $script:transferIndex++
        Write-Stage 'file-copy' 'done' "$source -> $destination" $timer.ElapsedMilliseconds
    } catch {
        Write-Stage 'file-copy' 'failed' "$source -> $destination; HRESULT=$($_.Exception.HResult)" $timer.ElapsedMilliseconds
        throw
    } finally {
        if ($inputFile) { $inputFile.Dispose() }
        # On failure, retain partial guest state for explicit reconciliation.
        # Rust kills the local bridge on deadline; that is not guest cancellation.
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
    if ($r.mode -cin @('compatibility', 'prepare')) {
        $checkingCompatibility = $true
        $identity = Invoke-Command -Session $session -ScriptBlock $readInstallationIdentity
        if ($r.mode -ceq 'compatibility') {
            @{ output = [string]$identity; worker_exit = 0 } | ConvertTo-Json -Compress
            return
        }
        # Rust supplies the release list used by admission and the guest worker.
        # Check in this very session, before creating guest directories or files.
        $os = $identity | ConvertFrom-Json
        if ($os.major -ne 10 -or $os.minor -ne 0 -or $os.workstation -ne $true -or $os.build -notin $r.windows_11_builds) {
            $failure = @{ stage=$null; component='GuestCompatibility'; cause='UnsupportedGuest'; code=$null } | ConvertTo-Json -Compress
            @{ output=('HGFAIL1:' + $failure); worker_exit=1 } | ConvertTo-Json -Compress
            exit 1
        }
        $checkingCompatibility = $false
    }
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
    # Validate existing leaves, but retain them until a complete authenticated
    # replacement is ready. Never copy over a pre-existing reparse/hard link.
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
            }
        }
    }
    Write-Stage 'bootstrap-leaves' 'done' $root $timer.ElapsedMilliseconds
    foreach ($a in $r.artifacts) {
        $destination = Join-Path $root $a.name
        $staged = Join-Path $root ($a.name + '-' + [guid]::NewGuid().ToString('N') + '.staged')
        Invoke-Command -Session $session -ArgumentList $staged -ScriptBlock {
            param($path)
            # CreateNew refuses collisions; the protected parent prevents ordinary
            # guest users substituting a link while Copy-Item opens this leaf.
            $file = [IO.File]::Open($path, 'CreateNew', 'Write', 'None')
            $file.Dispose()
            Set-Acl -LiteralPath $path -AclObject (Get-Acl -LiteralPath (Split-Path $path))
        }
        Copy-Logged (Join-Path $r.install $a.name) $staged $a.bytes
        $timer.Restart()
        Write-Stage 'bootstrap-publication' 'start' $a.name
        Invoke-Command -Session $session -ArgumentList $staged, $destination, $a.hash -ScriptBlock {
            param($staged, $destination, $expectedHash)
            $ErrorActionPreference = 'Stop'
            $item = Get-Item -LiteralPath $staged -Force
            if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe staged bootstrap leaf' }
            if ((Get-Acl -LiteralPath $staged).GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted staged bootstrap owner' }
            $handle = [IO.File]::Open($staged, 'Open', 'Read', 'Read')
            try {
                if ((Get-FileHash -LiteralPath $staged -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expectedHash) { throw 'Staged bootstrap integrity failure' }
            } finally { $handle.Dispose() }
            # Release the write/delete-denying validation handle only for atomic
            # publication inside the protected parent. There is no delete/copy gap.
            if (Test-Path -LiteralPath $destination) {
                $item = Get-Item -LiteralPath $destination -Force
                if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe bootstrap leaf' }
                if ((Get-Acl -LiteralPath $destination).GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted bootstrap leaf owner' }
                # A backup preserves the old bytes through ReplaceFile failure
                # modes; retain it on any uncertain publication/ACL/hash outcome.
                [IO.File]::Replace($staged, $destination, ($staged + '.previous'))
            } else {
                [IO.File]::Move($staged, $destination)
            }
            # Replace may retain the old leaf's ACL. Apply the protected parent's
            # exact ACL before the later locked-handle hash check and worker launch.
            Set-Acl -LiteralPath $destination -AclObject (Get-Acl -LiteralPath (Split-Path $destination))
            $handle = [IO.File]::Open($destination, 'Open', 'Read', 'Read')
            try {
                if ((Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expectedHash) { throw 'Published bootstrap integrity failure' }
                # Delete only our exact backup leaf after successful publication;
                # failures above leave it for explicit recovery, never rollback.
                [IO.File]::Delete($staged + '.previous')
            } finally { $handle.Dispose() }
        }
        Write-Stage 'bootstrap-publication' 'done' $a.name $timer.ElapsedMilliseconds
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
    if ($r.mode -eq 'verify') {
        # Metadata only: Verify never transfers driver payloads or rewrites prepared.json.
        Invoke-Command -Session $session -ArgumentList $bundle, $r.verification -ScriptBlock {
            param($path, $json)
            $leaf = Join-Path $path 'verification.json'
            if (Test-Path -LiteralPath $leaf) {
                $item = Get-Item -LiteralPath $leaf -Force
                if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe verification metadata' }
                if ((Get-Acl -LiteralPath $leaf).GetOwner([Security.Principal.SecurityIdentifier]).Value -notin @('S-1-5-18','S-1-5-32-544')) { throw 'Untrusted verification metadata' }
            }
            [IO.File]::WriteAllText($leaf, $json, [Text.UTF8Encoding]::new($false))
            Set-Acl -LiteralPath $leaf -AclObject (Get-Acl -LiteralPath $path)
        }
    }
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
            Copy-Logged $r.manifest.files[$i].source (Join-Path $bundle ([string]$i)) $r.manifest.files[$i].bytes
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
    Write-Transfer 'done' $transferTotal
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
            $reply = [Collections.Generic.List[string]]::new()
            & (Join-Path $root 'hyper-gpu-guest.exe') $mode $bundle ([string]$vendor) ([string]$device) 2>$null | ForEach-Object {
                $line = [string]$_
                if ($line.StartsWith('HGPREPARATION ', [StringComparison]::Ordinal)) {
                    $phase = $line.Substring(14)
                    if ($phase -cnotin @('checking-copied','installing','checking-installed','checking-guest','graphics')) { throw 'Invalid guest phase' }
                    # Remoting streams this fixed event immediately; no paths or error text.
                    @{ phase = $phase }
                } else {
                    if ($reply.Count -ge 8 -or $line.Length -gt 1048576) { throw 'Guest reply exceeds bound' }
                    $reply.Add($line)
                }
            }
            @{ output = ($reply -join "`n"); worker_exit = $LASTEXITCODE }
        } finally { foreach ($handle in $handles) { $handle.Dispose() }; if ($ownedMarker) { [IO.File]::Delete($marker) } }
    } | ForEach-Object {
        if ($_.phase) {
            [Console]::Error.WriteLine('HGPREPARATION ' + [string]$_.phase)
        } else { $_ }
    }
    Write-Stage 'guest-integrity-and-worker' 'done' $r.mode $timer.ElapsedMilliseconds
    @{ output = [string]$output.output; worker_exit = $output.worker_exit } | ConvertTo-Json -Compress
} catch {
    if ($checkingCompatibility) {
        $failure = @{ stage=$null; component='GuestCompatibility'; cause='Unavailable'; code=$null } | ConvertTo-Json -Compress
        @{ output=('HGFAIL1:' + $failure); worker_exit=1 } | ConvertTo-Json -Compress
        exit 1
    }
    Write-Stage 'transport' 'failed' "HRESULT=$($_.Exception.HResult); type=$($_.Exception.GetType().FullName)"
    [Console]::Error.WriteLine('Fixed guest transport failed; reconcile before retry.')
    @{ output = ''; transport_hresult = $_.Exception.HResult } | ConvertTo-Json -Compress
    exit 1
} finally {
    if ($session) {
        Write-Stage 'session-remove' 'start' 'PowerShell Direct'
        Remove-PSSession -Session $session
        Write-Stage 'session-remove' 'done' 'PowerShell Direct'
    }
    if ($diagnostic) { $diagnostic.Dispose() }
}
