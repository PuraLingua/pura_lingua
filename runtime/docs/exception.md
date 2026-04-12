# Exception

## ExceptionTable

It's a table stored in every Method, containing how to do when an exception encounters. Its entries are sorted(by [ExceptionTable::range.start](../src/type_system/method/exception_table.rs#L82)) in reverse order(e.g. [2, 1, 0])

Each entry contains a desired type of Exception, an optional filter, an optional catch pc, an optional finally pc, and an optional fault pc.

### Invoke order

* Finally
* Fault
* Catch

#### NOTE: Invocation will recover unless returned
