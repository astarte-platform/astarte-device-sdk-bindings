<!--
Copyright 2026 SECO Mind Srl

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

SPDX-License-Identifier: Apache-2.0
-->

# Astarte Device SDK Bindings

C-compatible FFI bindings for the [Astarte Device SDK](https://github.com/astarte-platform/astarte-device-sdk-rust) in Rust.

This crate provides C-compatible interfaces, data structures, and function exports (`extern "C"`) that allow non-Rust applications (such as C, C++, Python, or Java applications via FFI/JNI) to interact with an Astarte instance.

## Overview

Astarte is an open-source IoT platform that enables real-time device management, data streaming, and property synchronization. This library exposes the core functionality of the Rust Device SDK over C ABI boundaries:

- **Device Management**: Initialize, connect, disconnect, and free device handles (`NativeDeviceHandle`).
- **Data Transmission**: Send individual values (`device_handle_send_individual`) and object aggregates (`device_handle_send_object`).
- **Property Management**: Set (`device_handle_set_property`), unset (`device_handle_unset_property`), and query local property values (`device_handle_get_property`).
- **Event Reception**: Asynchronously receive device events (`device_handle_receive`).
- **Transport Protocols**: Supports both MQTT (direct Astarte connection with Pairing API) and gRPC (Astarte Message Hub).

## Architecture & Memory Ownership

The FFI layer uses explicit C-compatible structures and pointer conversion utilities:

1. **Device Handle**: `NativeDeviceHandle` is an opaque pointer managing an asynchronous Tokio runtime and device runtime state.
2. **Callbacks & User Data**: Asynchronous FFI operations take callback function pointers and an opaque `UserData` (`*mut c_void`) context pointer.
3. **Memory Cleanup**:
   - `device_handle_free`: Frees a `NativeDeviceHandle` and shuts down its Tokio runtime.
   - `device_handle_free_device_event`: Frees a `NativeDeviceEvent` received in event callbacks.
   - `device_handle_free_get_property`: Frees a `NativeOption<NativeDeviceData>` returned by property get callbacks.

## Supported Transports

Configuration is provided via `NativeDeviceConfig`:
- **MQTT Connection**: Requires `device_id`, `cred_secr`, `realm`, and `pairing_url`. Always available.
- **gRPC Connection**: Requires `message_hub_addr` for connecting through Astarte Message Hub. Gated behind the `message-hub` cargo feature, which is **enabled by default**.

## Cargo Features

| Feature | Default | Description |
| --- | --- | --- |
| `message-hub` | yes | Enables the gRPC/Message Hub transport, forwarding to the SDK's own `message-hub` feature |

Disabling it with `--no-default-features` removes the `Grpc` variant from `ConnectionConfig`/`NativeConnectionConfig`, along with the tonic/prost/`astarte-message-hub-proto` dependency tree.

> **Note**: the C header generated with `cbindgen` always covers the full, all-features ABI, since `cbindgen` parses the source rather than the compiled artifact. Keep `message-hub` enabled when building the library itself (the default) so that the `.so` matches that header.
