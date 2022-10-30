[CmdletBinding()]
param(
    [ValidateSet('simpleinterp', 'opinterp', 'opinterp2', 'opinterp3')]
    [Parameter(Mandatory, Position = 0)]
    [String] $Bin,

    [ValidateSet('mandelbrot', 'factor')]
    [Parameter(Mandatory, Position = 1)]
    [String] $SourceFile,

    [Switch] $BuildDebug,

    [Parameter(ValueFromPipeline)]
    [String] $Pipeline = $null
)

$local:ErrorActionPreference = 'Stop'

Push-Location $PSScriptRoot
try {
    $releaseArg
    if ($BuildDebug) {
        $releaseArg = ''
    } else {
        $releaseArg = '--release'
    }

    if ($Pipeline) {
        $Pipeline | cargo run $releaseArg --bin $Bin -- corpus/$SourceFile.bf
    } else {
        cargo run $releaseArg --bin $Bin -- corpus/$SourceFile.bf
    }
} finally {
    Pop-Location
}
