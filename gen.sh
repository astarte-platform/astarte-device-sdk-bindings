#!/bin/bash

cargo build --release

cargo run --bin uniffi-bindgen generate --library target/release/libastarte_device_sdk_bindings.so --language kotlin --out-dir out

cp target/release/libastarte_device_sdk_bindings.so astarte-kotlin-app/build/classes/kotlin/main/

cp out/uniffi/astarte_device_sdk_bindings/astarte_device_sdk_bindings.kt astarte-kotlin-app/src/main/kotlin/uniffi/astarte_bindings/
