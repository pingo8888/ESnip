param(
  [string]$Version,

  [string]$Notes,

  [string]$PrivateKeyPath = "$env:USERPROFILE\.tauri\esnip-updater.key",

  [switch]$SkipBump
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root

$VersionWasProvided = -not [string]::IsNullOrWhiteSpace($Version)

if (-not $VersionWasProvided) {
  $PackageJson = Get-Content -Raw -LiteralPath (Join-Path $Root "package.json") | ConvertFrom-Json
  $TauriConfig = Get-Content -Raw -LiteralPath (Join-Path $Root "src-tauri\tauri.conf.json") | ConvertFrom-Json
  $CargoToml = Get-Content -Raw -LiteralPath (Join-Path $Root "src-tauri\Cargo.toml")
  $CargoVersionMatch = [regex]::Match($CargoToml, '(?ms)^\[package\]\s+.*?^version\s*=\s*"([^"]+)"')

  if (-not $CargoVersionMatch.Success) {
    throw "Failed to read current version from src-tauri\Cargo.toml"
  }

  $Version = [string]$PackageJson.version
  $VersionSources = [ordered]@{
    "package.json" = $Version
    "src-tauri\tauri.conf.json" = [string]$TauriConfig.version
    "src-tauri\Cargo.toml" = $CargoVersionMatch.Groups[1].Value
  }

  $MismatchedVersions = $VersionSources.GetEnumerator() | Where-Object { $_.Value -ne $Version }
  if ($MismatchedVersions) {
    $Details = ($VersionSources.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ", "
    throw "Project version files do not match: $Details"
  }
}

$Version = $Version.Trim()

if ($Version -notmatch '^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') {
  throw "Invalid version: $Version"
}

if ([string]::IsNullOrWhiteSpace($Notes)) {
  $Notes = "ESnip $Version"
}

if ($VersionWasProvided -and -not $SkipBump) {
  node "$Root\bump-version.mjs" $Version
}

if (-not (Test-Path -LiteralPath $PrivateKeyPath)) {
  throw "Updater private key not found: $PrivateKeyPath"
}

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw -LiteralPath $PrivateKeyPath

pnpm run tauri build

$ReleaseDir = Join-Path $Root "release\$Version"
New-Item -ItemType Directory -Path $ReleaseDir -Force | Out-Null

$BundleDir = Join-Path $Root "src-tauri\target\release\bundle"
$NsisDir = Join-Path $BundleDir "nsis"
$Setup = Get-ChildItem -Path $NsisDir -Filter "ESnip_${Version}_x64-setup.exe" -File -ErrorAction Stop |
  Select-Object -First 1

if (-not $Setup) {
  throw "NSIS setup package not found for version $Version"
}

$SetupSig = Get-Item -LiteralPath "$($Setup.FullName).sig" -ErrorAction Stop

Copy-Item -LiteralPath $Setup.FullName -Destination $ReleaseDir -Force
Copy-Item -LiteralPath $SetupSig.FullName -Destination $ReleaseDir -Force

$Signature = (Get-Content -Raw -LiteralPath $SetupSig.FullName).Trim()
$SetupUrl = "https://github.com/pingo8888/ESnip/releases/download/$Version/$($Setup.Name)"

$Latest = [ordered]@{
  version = $Version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = $Signature
      url = $SetupUrl
    }
  }
}

$LatestPath = Join-Path $ReleaseDir "latest.json"
$Latest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $LatestPath -Encoding UTF8

Write-Host ""
Write-Host "Release files prepared:" -ForegroundColor Green
Get-ChildItem -Path $ReleaseDir -File | ForEach-Object {
  Write-Host " - $($_.FullName)"
}
Write-Host ""
Write-Host "Upload all files above to GitHub Release: $Version"
