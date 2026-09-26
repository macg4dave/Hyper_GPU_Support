[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$failures = [System.Collections.Generic.List[string]]::new()

Push-Location $repositoryRoot
try {
    $markdownFiles = @(& git ls-files --cached --others --exclude-standard -- '*.md')
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to enumerate tracked Markdown files (git exit $LASTEXITCODE)."
    }

    $inlineLinkPattern = [regex]'\[[^\]]+\]\((?<target>[^)]+)\)'
    foreach ($relativeFile in $markdownFiles) {
        $sourcePath = Join-Path $repositoryRoot $relativeFile
        $sourceDirectory = Split-Path -Parent $sourcePath
        $lineNumber = 0
        foreach ($line in Get-Content -LiteralPath $sourcePath) {
            $lineNumber++
            foreach ($match in $inlineLinkPattern.Matches($line)) {
                $target = $match.Groups['target'].Value.Trim().Trim('<', '>')
                if ($target -match '^(?:https?://|mailto:|#)') {
                    continue
                }

                # Ignore the anchor here; this check verifies that every local link
                # still names a tracked or generated repository path.
                $pathPart = ($target -split '#', 2)[0]
                if ([string]::IsNullOrWhiteSpace($pathPart)) {
                    continue
                }

                $decodedPath = [Uri]::UnescapeDataString($pathPart)
                $resolvedTarget = Join-Path $sourceDirectory $decodedPath
                if (-not (Test-Path -LiteralPath $resolvedTarget)) {
                    $failures.Add("${relativeFile}:${lineNumber}: missing local link target '$pathPart'")
                }
            }
        }
    }

    foreach ($prompt in Get-ChildItem -LiteralPath '.github/prompts' -Filter '*.prompt.md') {
        $lines = @(Get-Content -LiteralPath $prompt.FullName)
        if ($lines.Count -lt 4 -or $lines[0] -ne '---' -or $lines[3] -ne '---') {
            $failures.Add("$($prompt.FullName): invalid four-line prompt frontmatter")
        }
    }

    if ($failures.Count -gt 0) {
        throw ($failures -join [Environment]::NewLine)
    }

    & git diff --check
    if ($LASTEXITCODE -ne 0) {
        throw "git diff --check failed with exit code $LASTEXITCODE."
    }

    Write-Host "Documentation checks passed for $($markdownFiles.Count) repository Markdown files."
}
catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
finally {
    Pop-Location
}
