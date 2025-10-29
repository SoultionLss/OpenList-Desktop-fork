# Sign-Tauri-Windows.ps1
# Signs Windows Tauri application binaries with Certum certificate
# Uses authenticated SimplySign Desktop connection
# Supports multiple architectures (x86_64, aarch64)

param(
    [string]$BaseDirectory = "src-tauri/target",
    [string]$CertificateSHA1 = $env:CERTUM_CERTIFICATE_SHA1
)

Write-Host "=== TAURI WINDOWS BINARY SIGNING ==="
Write-Host "Base target directory: $BaseDirectory"
Write-Host "Certificate SHA1: $($CertificateSHA1.Substring(0,16))... (truncated)"
Write-Host ""

# Validate inputs
if (-not $CertificateSHA1) {
    Write-Host "ERROR: CERTUM_CERTIFICATE_SHA1 environment variable not provided"
    exit 1
}

if (-not (Test-Path $BaseDirectory)) {
    Write-Host "ERROR: Base target directory not found: $BaseDirectory"
    exit 1
}

# Detect all Windows architecture targets
Write-Host "Scanning for Windows architecture builds..."
$WindowsTargets = @(
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc"
)

$FoundTargets = @()
foreach ($target in $WindowsTargets) {
    $targetPath = Join-Path $BaseDirectory "$target\release"
    if (Test-Path $targetPath) {
        $FoundTargets += @{
            Target = $target
            Path = $targetPath
        }
        Write-Host "Found target: $target"
    }
}

if ($FoundTargets.Count -eq 0) {
    Write-Host "ERROR: No Windows target directories found"
    exit 1
}

Write-Host "Found $($FoundTargets.Count) architecture(s) to process"
Write-Host ""

# Find signtool - Auto-detect Windows SDK version
Write-Host "Locating signtool..."
$WindowsKitsBase = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"

if (-not (Test-Path $WindowsKitsBase)) {
    Write-Host "ERROR: Windows Kits base directory not found: $WindowsKitsBase"
    exit 1
}

# Scan for all SDK versions and sort by version number (descending)
Write-Host "Scanning Windows SDK versions in: $WindowsKitsBase"
$SDKVersions = Get-ChildItem -Path $WindowsKitsBase -Directory -ErrorAction SilentlyContinue | 
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
    Sort-Object { [version]$_.Name } -Descending

if ($SDKVersions.Count -eq 0) {
    Write-Host "ERROR: No Windows SDK version directories found"
    exit 1
}

Write-Host "Found $($SDKVersions.Count) SDK version(s): $($SDKVersions.Name -join ', ')"

# Try each SDK version until we find signtool
$SignTool = $null
foreach ($sdkVersion in $SDKVersions) {
    $signtoolPath = Join-Path $sdkVersion.FullName "x64\signtool.exe"
    Write-Host "Checking: $signtoolPath"
    
    if (Test-Path $signtoolPath) {
        $SignTool = $signtoolPath
        Write-Host "Found signtool: $SignTool (SDK version: $($sdkVersion.Name))"
        break
    }
}

if (-not $SignTool) {
    Write-Host "ERROR: signtool.exe not found in any SDK version directory"
    Write-Host "Searched in: $WindowsKitsBase"
    exit 1
}

# Process each architecture target
$TotalBinariesToSign = @()
$AllSignedCount = 0
$AllFailedCount = 0

foreach ($targetInfo in $FoundTargets) {
    $TargetDirectory = $targetInfo.Path
    $TargetName = $targetInfo.Target
    
    Write-Host "=== Processing $TargetName ==="
    Write-Host "Directory: $TargetDirectory"
    Write-Host ""
    
    # Find binaries to sign for this architecture
    Write-Host "Scanning for Windows binaries to sign..."
    $BinariesToSign = @()

    # Main Tauri executable
    $MainExecutable = Join-Path $TargetDirectory "openlist-desktop.exe"
    if (Test-Path $MainExecutable) {
        $BinariesToSign += $MainExecutable
        Write-Host "Found main executable: $MainExecutable"
    }

    # NSIS installer
    $NSISDir = Join-Path $TargetDirectory "bundle\nsis"
    if (Test-Path $NSISDir) {
        $NSISFiles = Get-ChildItem -Path $NSISDir -Filter "*.exe"
        foreach ($file in $NSISFiles) {
            $BinariesToSign += $file.FullName
            Write-Host "Found NSIS installer: $($file.FullName)"
        }
    }

    # MSI installer
    $MSIDir = Join-Path $TargetDirectory "bundle\msi"
    if (Test-Path $MSIDir) {
        $MSIFiles = Get-ChildItem -Path $MSIDir -Filter "*.msi"
        foreach ($file in $MSIFiles) {
            $BinariesToSign += $file.FullName
            Write-Host "Found MSI installer: $($file.FullName)"
        }
    }

    if ($BinariesToSign.Count -eq 0) {
        Write-Host "WARNING: No binaries found to sign for $TargetName"
        Write-Host "Contents of target directory:"
        Get-ChildItem -Path $TargetDirectory -Recurse -File | Select-Object FullName
        Write-Host ""
        continue
    }

    Write-Host ""
    Write-Host "Found $($BinariesToSign.Count) binaries to sign for $TargetName"
    Write-Host ""

    # Sign each binary
    $SignedCount = 0
    $FailedCount = 0

    foreach ($binary in $BinariesToSign) {
        $fileName = Split-Path $binary -Leaf
        Write-Host "=== Signing: $fileName ($TargetName) ==="
        
        # Get file size
        $fileSize = (Get-Item $binary).Length
        Write-Host "File size: $fileSize bytes"
        
        # Attempt signing with official Certum method
        Write-Host "Executing: signtool sign /sha1 `"***`" /tr http://time.certum.pl /td SHA256 /fd SHA256 /v `"$binary`""
        
        $signResult = & $SignTool sign /sha1 $CertificateSHA1 /tr "http://time.certum.pl" /td SHA256 /fd SHA256 /v $binary 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "SUCCESS: $fileName signed successfully"
            $SignedCount++
            
            # Verify the signature
            Write-Host "Verifying signature..."
            $verifyResult = & $SignTool verify /pa /v $binary 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "VERIFIED: Signature verification successful"
            } else {
                Write-Host "WARNING: Signature verification failed"
                Write-Host "Verify output: $verifyResult"
            }
            
        } else {
            Write-Host "FAILED: Signing failed for $fileName"
            Write-Host "Error output: $signResult"
            $FailedCount++
            
            # Try fallback method without /td parameter
            Write-Host "Attempting fallback method..."
            $fallbackResult = & $SignTool sign /sha1 $CertificateSHA1 /tr "http://time.certum.pl" /fd SHA256 /v $binary 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "SUCCESS: Fallback method worked for $fileName"
                $SignedCount++
                $FailedCount--
            } else {
                Write-Host "FAILED: Fallback method also failed"
                Write-Host "Fallback output: $fallbackResult"
            }
        }
        
        Write-Host ""
    }

    # Architecture summary
    Write-Host "=== $TargetName SIGNING SUMMARY ==="
    Write-Host "Total binaries: $($BinariesToSign.Count)"
    Write-Host "Successfully signed: $SignedCount"
    Write-Host "Failed to sign: $FailedCount"
    Write-Host ""
    
    $AllSignedCount += $SignedCount
    $AllFailedCount += $FailedCount
    $TotalBinariesToSign += $BinariesToSign
}

# Final summary
Write-Host "=== OVERALL SIGNING SUMMARY ==="
Write-Host "Architectures processed: $($FoundTargets.Count)"
Write-Host "Total binaries found: $($TotalBinariesToSign.Count)"
Write-Host "Successfully signed: $AllSignedCount"
Write-Host "Failed to sign: $AllFailedCount"

if ($AllFailedCount -eq 0) {
    Write-Host "ALL BINARIES SIGNED SUCCESSFULLY ACROSS ALL ARCHITECTURES!"
    Write-Host "Windows Tauri applications are ready for distribution"
    exit 0
} else {
    Write-Host "SOME BINARIES FAILED TO SIGN"
    Write-Host "Check the error messages above"
    exit 1
}
