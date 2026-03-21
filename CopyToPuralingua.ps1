param (
	[switch]$ShouldBuild
)

if ($ShouldBuild) {
	cargo build --workspace
}

$currentCwd = Get-Location

Set-Location $PSScriptRoot

$outDir = $Env:PURALINGUA_HOME
$outLibDir = Join-Path $outDir "Native" "Library"
$outExeDir = Join-Path $outDir "Native" "Executable"

Get-ChildItem -Path "./target/debug" -File `
| ForEach-Object -Process {
	$name = $_.Name
	if (
		($name.EndsWith(".dll") `
			-or $name.EndsWith(".lib") `
			-or $name.EndsWith(".pdb") `
			-or $name.EndsWith(".exp")) `
			-and $name.StartsWith("PuralinguaC")
	) {
		Copy-Item -Path $_ -Destination $outLibDir
	}
	else {
		if (
			$name.EndsWith(".exe")
		) {
			Copy-Item -Path $_ -Destination $outExeDir
		}
	}
}

Set-Location $currentCwd