$ErrorActionPreference = 'Stop'

$hostTriple = & rustc --print host-tuple
if ($LASTEXITCODE -ne 0) {
    throw "Rust-Zielplattform konnte nicht ermittelt werden."
}
$targetDirectory = Join-Path $PSScriptRoot 'target'
& cargo build --release --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') --target $hostTriple --target-dir $targetDirectory
if ($LASTEXITCODE -ne 0) {
    throw "Release-Build fehlgeschlagen (Exit-Code $LASTEXITCODE). Deploy wurde nicht aktualisiert."
}

$deployDirectory = Join-Path $PSScriptRoot 'deploy'
$null = New-Item -ItemType Directory -Path $deployDirectory -Force
Copy-Item -LiteralPath (Join-Path $targetDirectory "$hostTriple/release/timer-rs.exe") -Destination $deployDirectory -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'assets/ding.wav') -Destination $deployDirectory -Force

Write-Host "Deploy aktualisiert: $deployDirectory"