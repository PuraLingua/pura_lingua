# Puralingua Runtime

# WARNING
*NOT* every test passed when using commands like
```bash
cargo test --workspace
```
That's because tests rely on global variables, which are kept during different tests.
try using something like
```powershell
$failures = [System.Collections.Generic.List[string]]::new()
(cargo test -- --list).Split([System.Environment]::NewLine) | Where-Object {$_ -like "*: test"} | Foreach-Object -Process {
	$testCase = $_.Split(": ")[0]
	cargo test $testCase
	if ($LASTEXITCODE -ne 0) {
		$failures.Add($testCase)
	}
}
```