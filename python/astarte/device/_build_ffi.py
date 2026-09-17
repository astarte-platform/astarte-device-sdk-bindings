"""CFFI Out-of-line ABI builder for Astarte Device SDK bindings."""

import os
import re
from pathlib import Path
from cffi import FFI

ffibuilder = FFI()

_here = Path(__file__).resolve().parent
_root_dir = _here.parents[2]
_header_path = _root_dir / "target" / "header.h"

if not _header_path.exists():
    _env_header = os.getenv("ASTARTE_SDK_HEADER_PATH")
    if _env_header:
        _header_path = Path(_env_header)

if _header_path.exists():
    with open(_header_path, "r") as f:
        with_includes = f.read()
        definitions = re.sub(r"#include.+", "", with_includes)
        ffibuilder.cdef(definitions)

ffibuilder.cdef("""
    extern "Python" void get_property_cbk(const struct NativeStringResult_NativeOption_NativeDeviceData *native_res, UserData user_data);
    extern "Python" void send_cbk(const struct NativeStringResult_bool *native_res, UserData user_data);
    extern "Python" void receive_cbk(const struct NativeStringResult_NativeManuallyDrop_NativeDeviceEvent *native_res, UserData user_data);
    extern "Python" void disconnect_cbk(const struct NativeStringResult_bool *native_res, UserData user_data);
    extern "Python" void connect_cbk(const struct NativeStringResult_bool *native_result, UserData user_data);
    extern "Python" void loop_cbk(const struct NativeStringResult_bool *native_result, UserData user_data);
""")

ffibuilder.set_source("_astarte_cffi",
    """
        #include <>
    """, libraries = ["libastarte_device_sdk_bindings"])

if __name__ == "__main__":
    ffibuilder.compile(verbose=True)
