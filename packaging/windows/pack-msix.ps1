[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $IdentityName,
    [Parameter(Mandatory)] [string] $Publisher,
    [Parameter(Mandatory)] [string] $PublisherDisplayName,
    [string] $Binary,
    [string] $OutputPath
)

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $Binary) { $Binary = Join-Path $project 'src-tauri\target\release\clock-study.exe' }
if (-not $OutputPath) { $OutputPath = Join-Path $project 'artifacts\ClockStudy-store-x64.msix' }
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$output = [IO.Path]::GetFullPath($OutputPath)
[xml] $manifest = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'AppxManifest.xml') -Raw -Encoding UTF8
$manifest.Package.Identity.SetAttribute('Name', $IdentityName)
$manifest.Package.Identity.SetAttribute('Publisher', $Publisher)
$manifest.Package.Properties.PublisherDisplayName = $PublisherDisplayName
$projectVersion = (Get-Content -LiteralPath (Join-Path $project 'package.json') -Raw | ConvertFrom-Json).version
if ($projectVersion -notmatch '^\d+\.\d+\.\d+$') { throw 'Use uma versão numérica de três partes em package.json.' }
$manifest.Package.Identity.SetAttribute('Version', "$projectVersion.0")

$makeAppx = Get-Command MakeAppx.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
if (-not $makeAppx) {
    $sdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    if (Test-Path -LiteralPath $sdk) {
        $makeAppx = Get-ChildItem -LiteralPath $sdk -Directory |
            Where-Object Name -Match '^10\.\d+\.\d+\.\d+$' |
            Sort-Object { [version] $_.Name } -Descending |
            ForEach-Object { Join-Path $_.FullName 'x64\MakeAppx.exe' } |
            Where-Object { Test-Path -LiteralPath $_ } |
            Select-Object -First 1
    }
}
if (-not $makeAppx) { throw 'MakeAppx.exe não foi encontrado no SDK do Windows.' }

$outDirectory = Split-Path -Parent $output
$staging = Join-Path $outDirectory ('msix-staging-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $staging -Force
$manifest.Save((Join-Path $staging 'AppxManifest.xml'))
Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $staging 'ClockStudy.exe')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Assets') -Destination $staging -Recurse

& $makeAppx pack /o /d $staging /p $output
if ($LASTEXITCODE -ne 0) { throw "MakeAppx falhou com código $LASTEXITCODE." }
Write-Output "Pacote para submissão à Store: $output"
