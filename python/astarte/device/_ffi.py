"""FFI loading and initialization for Astarte Device SDK bindings."""

from __future__ import annotations

import os
import re
from pathlib import Path
from cffi import FFI

_here = Path(__file__).resolve().parent
_root_dir = _here.parents[2]

ffi = FFI()

_header_path = _root_dir / "target" / "header.h"
_env_header = os.getenv("ASTARTE_SDK_HEADER_PATH")
if _env_header:
    _header_path = Path(_env_header)

if _header_path.exists():
    with open(_header_path, "r") as f:
        definitions = re.sub(r"#include.+", "", f.read())
        ffi.cdef(definitions)


def _find_library() -> str:
    """Find the Astarte device SDK shared library path."""
    env_path = os.getenv("ASTARTE_SDK_LIB_PATH")
    if env_path and Path(env_path).exists():
        return str(env_path)

    lib_name = "libastarte_device_sdk_bindings.so"
    debug_target = _root_dir / "target" / "debug" / lib_name
    release_target = _root_dir / "target" / "release" / lib_name

    if debug_target.exists():
        return str(debug_target)
    elif release_target.exists():
        return str(release_target)
    else:
        raise FileNotFoundError(
            f"Shared library {lib_name} not found in {debug_target} or {release_target}. "
            "Please build the Rust project (`cargo build`) or set ASTARTE_SDK_LIB_PATH."
        )


_lib = None


def get_lib():
    """Get or open the CFFI handle to the native shared library."""
    global _lib
    if _lib is None:
        lib_path = _find_library()
        _lib = ffi.dlopen(lib_path)
    return _lib


__all__ = ["ffi", "get_lib"]
