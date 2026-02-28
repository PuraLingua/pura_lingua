# Puralingua Runtime

# WARNING
*NOT* every test passed when using commands like
```bash
cargo test --workspace
```
That's because tests rely on global variables, which are kept during different tests.
try using something like
```powershell
function Get-AllRustTest {
	(cargo test -- --list).Split([System.Environment]::NewLine) `
	| Where-Object {$_ -like "*: test"} `
	| Foreach-Object -Process {$_.Split(": ")[0]}
}
$failures = [System.Collections.Generic.List[string]]::new()
Get-AllRustTest | Foreach-Object -Process {
	cargo test $_
	if ($LASTEXITCODE -ne 0) {
		$failures.Add($_)
	}
}
```