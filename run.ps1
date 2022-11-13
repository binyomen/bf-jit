[CmdletBinding()]
param(
    [ValidateSet('simpleinterp', 'opinterp', 'opinterp2', 'opinterp3', 'simplejit', 'opjit')]
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
    if ($BuildDebug) {
        $Pipeline | cargo run --bin $Bin -- corpus/$SourceFile.bf
    } else {
        $Pipeline | cargo run --release --bin $Bin -- corpus/$SourceFile.bf
    }
} finally {
    Pop-Location
}
