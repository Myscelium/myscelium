# Build and test instructions

> [!WARNING]
> This is the current development workflow, not a stable release contract.
> Clean-machine builds and the complete test suite are still being stabilized.

## Requirements

- Git with submodule support
- Python and `venv`
- A current Rust toolchain with Cargo
- A native C/C++ build toolchain for your platform
- `setuptools`, `wheel`, `setuptools-rust`, and `pytest`

The repository contains a historical Windows CPython 3.10 `.pyd` artifact. Do
not assume that binary works with another operating system or Python version;
build the extension locally instead.

## Clone

```bash
git clone --recurse-submodules https://github.com/Myscelium/myscelium.git
cd myscelium
git submodule update --init --recursive
```

## Create a virtual environment

From the repository root:

```bash
cd Myscelium
python -m venv .venv
```

Activate it on Linux or macOS:

```bash
source .venv/bin/activate
```

Activate it on Windows PowerShell:

```powershell
.venv\Scripts\Activate.ps1
```

Install the current runtime, test, and build dependencies:

```bash
python -m pip install --upgrade pip
python -m pip install -r requirements.txt
python -m pip install setuptools wheel setuptools-rust pytest-timeout psutil simplejson pydantic
```

These dependencies are listed explicitly because package metadata is still
being prepared to install the complete runtime dependency set automatically.

## Editable development build

```bash
python -m pip install -e .
```

## Build a wheel

```bash
python setup.py bdist_wheel
```

The wheel is written to `dist/`. Install a specific generated wheel with:

```bash
python -m pip install --force-reinstall dist/<wheel-file>.whl
```

## Run the Python integration suite

From `Myscelium/`:

```bash
python -m pytest -v -s tests/test_myscelium.py
```

The suite starts local host/client processes and may create runtime state under
the test directories. Run it in an isolated checkout.

## Run Rust checks directly

Backend submodule:

```bash
cargo test --manifest-path OxidizedMysceliumCore/Cargo.toml --workspace --locked
```

Python bridge:

```bash
cargo test --manifest-path rust/Cargo.toml --locked
```

## Test result viewer

If a test run produced the expected history data:

```bash
streamlit run tests/History/interface.py
```
