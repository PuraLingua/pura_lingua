# Puralingua Runtime

# WARNING
*NOT* every test passed when using commands like
```bash
cargo test --workspace
```
tests might fail:
* stdlib::System::DynamicLibrary::tests::simple_dynamic_lib_test
* type_system::method::tests::test_normal_f
* virtual_machine::cpu::tests::dynamic_message_box
* virtual_machine::cpu::tests::dynamic_non_purus_call
* virtual_machine::cpu::tests::test_call_stack