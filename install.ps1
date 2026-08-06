[CmdletBinding()]
param(
    [string]$Repository = $(if ($env:VULCANUM_REPOSITORY) { $env:VULCANUM_REPOSITORY } else { "EzyGang/vulcanum" }),
    [string]$InstallDir = $(if ($env:VULCANUM_INSTALL_DIR) { $env:VULCANUM_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Vulcanum\bin" }),
    [string]$Version = $(if ($env:VULCANUM_VERSION) { $env:VULCANUM_VERSION } else { "latest" })
)

$ErrorActionPreference = "Stop"
$target = "x86_64-pc-windows-msvc"

function Resolve-LatestTag {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest" -Headers @{ Accept = "application/vnd.github+json" }

    if ([string]::IsNullOrWhiteSpace($release.tag_name)) {
        throw "No published Vulcanum release was found."
    }

    return $release.tag_name
}

function Get-ReleaseTag {
    if ($Version -eq "latest") {
        return Resolve-LatestTag
    }

    if ($Version.StartsWith("v")) {
        return $Version
    }

    return "v$Version"
}

function Verify-Checksum([string]$ArchivePath, [string]$ChecksumPath, [string]$ArchiveName) {
    $checksumLine = Get-Content -LiteralPath $ChecksumPath -Raw
    if ($checksumLine -notmatch "^(?<checksum>[A-Fa-f0-9]{64})\s+\*?(?<filename>[^\r\n]+)\s*$") {
        throw "The release checksum has an invalid format."
    }

    if ($Matches.filename -ne $ArchiveName) {
        throw "The release checksum does not identify $ArchiveName."
    }

    $actualChecksum = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash
    if ($actualChecksum -ne $Matches.checksum) {
        throw "Checksum verification failed."
    }
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Only 64-bit Windows is supported."
}

$tag = Get-ReleaseTag
$archiveName = "vulcanum-$target.zip"
$releaseUrl = "https://github.com/$Repository/releases/download/$tag"
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "vulcanum-$([guid]::NewGuid())"

try {
    New-Item -ItemType Directory -Path $tempDir | Out-Null
    $archivePath = Join-Path $tempDir $archiveName
    $checksumPath = "$archivePath.sha256"

    Write-Host "Downloading Vulcanum $tag for $target..."
    Invoke-WebRequest -Uri "$releaseUrl/$archiveName" -OutFile $archivePath
    Invoke-WebRequest -Uri "$releaseUrl/$archiveName.sha256" -OutFile $checksumPath
    Verify-Checksum $archivePath $checksumPath $archiveName

    Expand-Archive -LiteralPath $archivePath -DestinationPath $tempDir
    $binaryPath = Join-Path $tempDir "vulcanum.exe"
    if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
        throw "The release archive does not contain vulcanum.exe."
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $InstallDir "vulcanum.exe") -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = if ($userPath) { $userPath -split ";" } else { @() }
    if ($pathEntries -notcontains $InstallDir) {
        $updatedUserPath = (@($pathEntries | Where-Object { $_ }) + $InstallDir) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $updatedUserPath, "User")
        $env:Path = "$InstallDir;$env:Path"
        Write-Host "Added $InstallDir to your user PATH."
    }

    Write-Host "Installed vulcanum.exe to $InstallDir"
    Write-Host "Open a new PowerShell session before running vulcanum."
}
finally {
    if (Test-Path -LiteralPath $tempDir) {
        Remove-Item -LiteralPath $tempDir -Recurse -Force
    }
}
