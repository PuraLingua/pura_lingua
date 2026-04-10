$DEFINITIONS = @(
	"UInt8",
	"UInt16",
	"UInt32",
	"UInt64",
	"USize",

	"Int8",
	"Int16",
	"Int32",
	"Int64",
	"ISize"
)

$template = Get-Content -Path (Join-Path -Path $PSScriptRoot -ChildPath "src", "System", "_Integer.tpl.rs")

foreach ($def in $DEFINITIONS) {
	$content = $template.Replace("[<NAME>]", $def)
	Out-File -FilePath (Join-Path -Path $PSScriptRoot -ChildPath "src", "System", "Integers", "$def.rs") -InputObject $content -Encoding utf8NoBOM
}