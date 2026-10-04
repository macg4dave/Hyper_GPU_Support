# Privilege: non-elevated (dot-sourced helper; no privileged effects).
Set-StrictMode -Version Latest

function Import-ProjectConfiguration {
    [CmdletBinding()]
    param(
        [string] $Path = (Join-Path $PSScriptRoot '..\..\config\project.toml')
    )

    $resolvedPath = (Resolve-Path -LiteralPath $Path -ErrorAction Stop).Path
    $values = [Collections.Generic.Dictionary[string, object]]::new(
        [StringComparer]::Ordinal
    )
    $section = ''
    $lineNumber = 0
    foreach ($rawLine in Get-Content -LiteralPath $resolvedPath -Encoding UTF8) {
        $lineNumber++
        $line = $rawLine.Trim()
        if ($line.Length -eq 0 -or $line.StartsWith('#')) {
            continue
        }
        if ($line -match '^\[([A-Za-z0-9_.-]+)\]$') {
            $section = $Matches[1]
            continue
        }
        if ($line -notmatch '^([A-Za-z0-9_.-]+)\s*=\s*(.+)$') {
            throw "Unsupported project TOML syntax at ${resolvedPath}:${lineNumber}."
        }
        $key = if ($section.Length -eq 0) { $Matches[1] } else { "$section.$($Matches[1])" }
        $encodedValue = $Matches[2].Trim()
        if ($encodedValue -match "^'''(.*)'''$") {
            $value = $Matches[1]
        } elseif ($encodedValue -match '^"(?:[^"\\]|\\.)*"$') {
            try {
                $value = $encodedValue | ConvertFrom-Json -ErrorAction Stop
            } catch {
                throw "Invalid quoted project TOML value at ${resolvedPath}:${lineNumber}."
            }
        } elseif ($encodedValue -match '^\[(?:\s*"(?:[^"\\]|\\.)*"\s*,?)*\]$') {
            try {
                $value = @($encodedValue | ConvertFrom-Json -ErrorAction Stop)
            } catch {
                throw "Invalid string-array project TOML value at ${resolvedPath}:${lineNumber}."
            }
        } elseif ($encodedValue -match '^\d+$') {
            $value = [uint64]::Parse($encodedValue, [Globalization.CultureInfo]::InvariantCulture)
        } elseif ($encodedValue -eq 'true' -or $encodedValue -eq 'false') {
            $value = $encodedValue -eq 'true'
        } else {
            throw "Unsupported project TOML value at ${resolvedPath}:${lineNumber}."
        }
        if ($values.ContainsKey($key)) {
            throw "Duplicate project configuration key '$key'."
        }
        $values.Add($key, $value)
    }
    if (-not $values.ContainsKey('schema') -or $values['schema'] -ne 1) {
        throw 'Project configuration schema must be 1.'
    }
    return $values
}

function Get-ProjectConfigurationValue {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [Collections.Generic.Dictionary[string, object]] $Configuration,

        [Parameter(Mandatory)]
        [string] $Key
    )

    if (-not $Configuration.ContainsKey($Key)) {
        throw "Required project configuration key '$Key' is missing."
    }
    return $Configuration[$Key]
}
