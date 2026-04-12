$ErrorActionPreference = 'Continue'

$repo = 'Spoakk/cli'
$bin = 'spoak.exe'
$installDir = Join-Path $env:USERPROFILE '.spoak\bin'
$hasError = $false

function Write-Step {
    param($msg)
    Write-Host "  $msg" -ForegroundColor Cyan
}

function Write-Ok {
    param($msg)
    Write-Host "  $msg" -ForegroundColor Green
}

function Write-Err {
    param($msg)
    Write-Host "  $msg" -ForegroundColor Red
}

function Write-Warn {
    param($msg)
    Write-Host "  $msg" -ForegroundColor Yellow
}

function Wait-ForKey {
    Write-Host ''
    Write-Host '  Press Enter to exit...' -ForegroundColor DarkGray
    $null = Read-Host
}

Write-Host ''
Write-Host '  Spoak CLI Installer' -ForegroundColor Magenta
Write-Host '  ---------------------------------' -ForegroundColor DarkGray
Write-Host ''

Write-Step 'Fetching latest release from GitHub...'
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers @{ 'User-Agent' = 'spoak-installer' } -TimeoutSec 30 -ErrorAction Stop
    
    if (-not $release) {
        throw 'Empty response from GitHub API'
    }
    
    $tag = $release.tag_name
    if (-not $tag) {
        throw 'No tag_name found in release'
    }
    
    Write-Ok "Found release: $tag"
} catch {
    Write-Err 'Failed to fetch release information'
    $errMsg = $_.Exception.Message
    Write-Err "Error: $errMsg"
    
    if ($errMsg -match '404') {
        Write-Warn 'Repository not found or no releases available'
    } elseif ($errMsg -match '403') {
        Write-Warn 'GitHub API rate limit exceeded. Please try again later'
    } elseif ($errMsg -match 'timeout') {
        Write-Warn 'Request timed out. Please check your connection'
    }
    
    $hasError = $true
    Wait-ForKey
    exit 1
}

Write-Step "Looking for $bin..."
try {
    $asset = $release.assets | Where-Object { $_.name -eq $bin } | Select-Object -First 1
    
    if (-not $asset) {
        Write-Err "Asset not found in release $tag"
        Write-Warn 'Available assets:'
        $release.assets | ForEach-Object { Write-Host "    - $($_.name)" -ForegroundColor DarkGray }
        throw 'Required asset not found'
    }
    
    $url = $asset.browser_download_url
    $sizeInMB = [math]::Round($asset.size / 1MB, 2)
    Write-Ok "Found $bin ($sizeInMB MB)"
} catch {
    Write-Err 'Asset verification failed'
    $hasError = $true
    Wait-ForKey
    exit 1
}

Write-Step 'Creating installation directory...'
try {
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force -ErrorAction Stop | Out-Null
        Write-Ok "Created: $installDir"
    } else {
        Write-Ok "Directory exists: $installDir"
    }
} catch {
    Write-Err 'Failed to create installation directory'
    $errMsg = $_.Exception.Message
    Write-Err "Error: $errMsg"
    Write-Warn 'Please check folder permissions'
    $hasError = $true
    Wait-ForKey
    exit 1
}

$dest = Join-Path $installDir $bin

if (Test-Path $dest) {
    Write-Warn 'Existing installation found'
    Write-Step 'Removing old version...'
    try {
        Remove-Item $dest -Force -ErrorAction Stop
        Write-Ok 'Old version removed'
    } catch {
        Write-Err 'Failed to remove old version'
        $errMsg = $_.Exception.Message
        Write-Err "Error: $errMsg"
        Write-Warn 'Please close any running spoak processes and try again'
        $hasError = $true
        Wait-ForKey
        exit 1
    }
}

Write-Step "Downloading $bin..."
try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing -TimeoutSec 120 -ErrorAction Stop
    $ProgressPreference = 'Continue'
    
    if (-not (Test-Path $dest)) {
        throw 'Downloaded file not found'
    }
    
    $fileItem = Get-Item $dest
    $downloadedSize = [math]::Round($fileItem.Length / 1MB, 2)
    Write-Ok "Download successful ($downloadedSize MB)"
} catch {
    Write-Err 'Download failed'
    $errMsg = $_.Exception.Message
    Write-Err "Error: $errMsg"
    
    if ($errMsg -match '404') {
        Write-Warn 'File not found on server'
    } elseif ($errMsg -match 'timeout') {
        Write-Warn 'Download timed out. Please try again'
    } elseif ($errMsg -match 'SSL') {
        Write-Warn 'SSL/TLS error. Please check your system date/time'
    }
    
    $hasError = $true
    Wait-ForKey
    exit 1
}

Write-Step 'Verifying file...'
try {
    $fileInfo = Get-Item $dest -ErrorAction Stop
    if ($fileInfo.Length -lt 1KB) {
        throw 'Downloaded file is too small (possibly corrupted)'
    }
    Write-Ok 'File integrity check passed'
} catch {
    Write-Err 'File verification failed'
    $hasError = $true
    Wait-ForKey
    exit 1
}

Write-Step 'Adding to PATH...'
try {
    $currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User')
    $pathParts = $currentPath -split ';'
    $alreadyInPath = $pathParts | Where-Object { $_.Trim() -ieq $installDir }

    if (-not $alreadyInPath) {
        if ($currentPath) {
            $newPath = $currentPath + ';' + $installDir
        } else {
            $newPath = $installDir
        }
        [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User')
        Write-Ok 'Added to PATH'
    } else {
        Write-Ok 'Already in PATH'
    }
} catch {
    Write-Err 'Failed to update PATH'
    $errMsg = $_.Exception.Message
    Write-Err "Error: $errMsg"
    Write-Warn 'You may need to add to PATH manually'
    $hasError = $true
}

try {
    $env:PATH = $env:PATH + ';' + $installDir
} catch {
    Write-Warn 'Could not update current session PATH'
}

# Test installation
Write-Step 'Testing installation...'
try {
    $testResult = & $dest --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Ok 'Installation test passed'
    } else {
        Write-Warn 'Installation test returned non-zero exit code'
    }
} catch {
    Write-Warn 'Could not test installation'
}

Write-Host ''
if (-not $hasError) {
    Write-Host '  ✓ Spoak CLI installed successfully!' -ForegroundColor Green
    Write-Host ''
    Write-Host '  Next steps:' -ForegroundColor Cyan
    Write-Host '    1. Open a new terminal window' -ForegroundColor DarkGray
    Write-Host '    2. Run: spoak' -ForegroundColor DarkGray
    Write-Host ''
} else {
    Write-Host '  ✗ Installation completed with errors' -ForegroundColor Yellow
    Write-Host ''
    Write-Host '  Please review the errors above and try again' -ForegroundColor DarkGray
    Write-Host ''
}

Wait-ForKey
