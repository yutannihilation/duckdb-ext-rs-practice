# A Practice of Implementing A DuckDB Extension By Rust

## Cloning

Clone the repo with submodules

```shell
git clone --recurse-submodules https://github.com/yutannihilation/duckdb-ext-rs-practice
```

## Building

After making sure the dependencies (Python3, venv, make, git) are installed, building is a two-step process. Firstly run:
```shell
make configure
```
This will ensure a Python venv is set up with DuckDB and DuckDB's test runner installed. Additionally, depending on configuration,
DuckDB will be used to determine the correct platform for which you are compiling.

Then, to build the extension run:
```shell
make debug
```
This delegates the build process to cargo, which will produce a shared library in `target/debug/<shared_lib_name>`. After this step, 
a script is run to transform the shared library into a loadable extension by appending a binary footer. The resulting extension is written
to the `build/debug` directory.

To create optimized release binaries, simply run `make release` instead.

### Windows

On Windows, there are several things you need to be careful about:

- It seems this Makefile doesn't work on Powershell. Launch some other shell like Git Bash.
- If Python3 is installed other name than `python3`, you need to set `PYTHON_BIN` envvar.
  ```shell
  export PYTHON_BIN=python
  make configure
  make debug
  ```

## Testing
This extension uses the DuckDB Python client for testing. This should be automatically installed in the `make configure` step.
The tests themselves are written in the SQLLogicTest format, just like most of DuckDB's tests. A sample test can be found in
`test/sql/<extension_name>.test`. To run the tests using the *debug* build:

```shell
make test_debug
```

or for the *release* build:
```shell
make test_release
```

### Running the extension
To run the extension code, start `duckdb` with `-unsigned` flag. This will allow you to load the local extension file.

```sh
duckdb -unsigned
```

After loading the extension by the file path, you can use the functions provided by the extension (in this case, `hello_table()` and `hello_scalar()`).

```sql
LOAD './build/debug/extension/rusty_quack/rusty_quack.duckdb_extension';

SELECT * FROM hello_table('Jane');
```

```
┌─────────────────────┐
│       column0       │
│       varchar       │
├─────────────────────┤
│ Rusty Quack Jane 🐥 │
└─────────────────────┘
```

```sql
SELECT hello_scalar(col1) FROM (VALUES (1), (22)) tbl1(col1);
```

```
┌────────────────────┐
│ hello_scalar(col1) │
│       int32        │
├────────────────────┤
│                  2 │
│                 44 │
└────────────────────┘
```

### Version switching 
Testing with different DuckDB versions is really simple:

First, run 
```
make clean_all
```
to ensure the previous `make configure` step is deleted.

Then, run 
```
DUCKDB_TEST_VERSION=v1.1.2 make configure
```
to select a different duckdb version to test with

Finally, build and test with 
```
make debug
make test_debug
```

### Known issues
This is a bit of a footgun, but the extensions produced by this template may (or may not) be broken on windows on python3.11 
with the following error on extension load:
```shell
IO Error: Extension '<name>.duckdb_extension' could not be loaded: The specified module could not be found
```
This was resolved by using python 3.12