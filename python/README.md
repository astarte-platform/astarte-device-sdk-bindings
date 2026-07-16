# Astarte Device SDK Python Bindings

Python bindings for `astarte-device-sdk` built with CFFI.

## Installation

Install in editable mode:
```bash
pip install -e python/
```

## Structure

- `astarte/device/`
  - `data.py`: Data and Value representations (`DeviceData`, `DeviceObject`, `DeviceProperty`, `DeviceEvent`).
  - `config.py`: Device configuration (`DeviceConfig`).
  - `device.py`: Core client class (`Device`).
  - `callbacks.py`: FFI callback definitions and asyncio Future wrappers.
  - `exceptions.py`: Custom error hierarchy.
  - `_ffi.py`: CFFI shared library loader.
- `examples/send_example.py`: Runnable usage demonstration script.
- `tests/`: Package test suite.

## Running Tests

```bash
python3 -m unittest discover -s python/tests
```

## Running Example

Build and generate c headers and then run the example:
```bash
cargo b
cbindgen --config ./cbindgen.toml --crate astarte-device-sdk-bindings --output target/header.h
```
then in the python directory you can run
```bash
hatch run examples:python examples/send_example.py
```
