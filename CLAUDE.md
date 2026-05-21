# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Does

This project exposes the [Astarte Device SDK (Rust)](https://github.com/astarte-platform/astarte-device-sdk-rust) to Java via a native FFI bridge built with [BoltFFI](https://www.boltffi.dev/docs/getting-started). The Rust crate compiles to a shared library; BoltFFI generates the JNI glue layer and Java wrapper classes automatically.

## Commands

### Rust (core library)

```sh
cargo build          # build the Rust library (cdylib + staticlib)
cargo check          # type-check without linking
```

### Generate Java bindings

`JAVA_HOME` must be set before running this command:

```sh
export JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64"
boltffi pack java    # compile Rust, generate JNI glue, .so files, and Java sources into dist/java/
```

Output lands in `dist/java/`:
- `libastarte_device_sdk_bindings.so` — the core Rust shared library
- `libastarte_device_sdk_bindings_jni.so` — the JNI wrapper
- `org/astarte/device/sdk/bindings/` — generated Java sources

**Post-generation fix:** BoltFFI generates `Double.compare`, `Double.hashCode`, `Boolean.hashCode`,
and `Integer.hashCode` as static calls inside inner classes of the same name, which shadows
`java.lang.Double` / `java.lang.Boolean` / `java.lang.Integer`. After each `boltffi pack java` run,
qualify those calls with the fully-qualified type name in `dist/java/…/AstarteVal.java`:

```java
// AstarteVal.Double.hashCode()
result = 31 * result + java.lang.Double.hashCode(value);
return java.lang.Double.compare(this.value, other.value) == 0;

// AstarteVal.Boolean.hashCode()
result = 31 * result + java.lang.Boolean.hashCode(value);

// AstarteVal.Integer.hashCode()
result = 31 * result + java.lang.Integer.hashCode(value);
```

### Wire up and run the Java app (Gradle 8)

```sh
cp dist/java/libastarte_device_sdk_bindings_jni.so astarte-sdk-java/app/libs/native/
mkdir -p astarte-sdk-java/app/target/debug
cp dist/java/libastarte_device_sdk_bindings.so astarte-sdk-java/app/target/debug/
cp -r dist/java/org astarte-sdk-java/app/src/main/java/

cd astarte-sdk-java
gradle build
gradle run
```

The JVM is launched with `-Djava.library.path=libs/native` (set in `app/build.gradle`), so the JNI `.so` must be in `app/libs/native/`.

## Architecture

```
src/lib.rs                  ← Rust FFI surface (BoltFFI macros)
interfaces/                 ← Astarte interface JSON definitions
dist/java/                  ← BoltFFI-generated artifacts (JNI glue + Java wrappers)
astarte-sdk-java/           ← Gradle multi-project: app / list / utilities
  app/src/main/java/org/
    astarte/device/sdk/bindings/   ← copied from dist/java after boltffi pack
    example/app/App.java           ← demo entry point
```

### BoltFFI macro conventions in `src/lib.rs`

| Macro / attribute | Purpose |
|---|---|
| `#[data]` | Marks a `struct` or `enum` as a plain data type — generates Java POJO / sealed class |
| `#[boltffi::error]` | Marks an `enum` as the error type — surfaces as a Java checked exception (`SdkError`) |
| `#[export]` on a `trait` | Generates a Java `interface` that the caller implements (callback / listener pattern) |
| `#[export]` on an `impl` block | Generates a Java class wrapping the Rust object with matching methods |

### Key types

- `AstarteConfig` — plain data struct holding MQTT credentials and pairing URL
- `AstarteDevice` — the main object; wraps `DeviceClient<Mqtt<MemoryStore>>` and a `tokio::Runtime` so blocking FFI calls can drive async SDK operations
- `AstarteVal` — FFI-safe enum covering all 13 representable `AstarteData` variants (see table below); `BinaryBlobArray` is omitted because nested `Vec<Vec<u8>>` is unsupported by BoltFFI's `#[data]` macro
- `ObjectEntry` — plain data struct (`key: String`, `value: AstarteVal`) used to represent one field of an Object-aggregation event; passed as `Vec<ObjectEntry>` in `EventListener::on_object_received`
- `EventListener` — callback interface implemented by Java callers to receive connect/disconnect/data events; has six methods: `on_connected`, `on_disconnected`, `on_data_received`, `on_object_received`, `on_property_received`, `on_property_unset`
- `SdkError` — FFI error enum surfaced as a Java exception

### AstarteVal variants

| Variant | Inner field type | Notes |
|---|---|---|
| `Double` | `f64` | Validated (no NaN/Inf/subnormal) via `SdkDouble::try_from` |
| `Integer` | `i32` | |
| `Boolean` | `bool` | |
| `LongInteger` | `i64` | |
| `IString` | `String` | Named `IString` to avoid clash with the Rust `String` type |
| `BinaryBlob` | `Vec<u8>` | |
| `DateTime` | `i64` | Milliseconds since Unix epoch |
| `DoubleArray` | `Vec<f64>` | Each element validated via `SdkDouble::try_from` |
| `IntegerArray` | `Vec<i32>` | |
| `BooleanArray` | `Vec<bool>` | |
| `LongIntegerArray` | `Vec<i64>` | |
| `StringArray` | `Vec<String>` | |
| `DateTimeArray` | `Vec<i64>` | Milliseconds since Unix epoch |

Conversions use standard Rust traits:
- `TryFrom<AstarteData> for AstarteVal` (SDK → FFI, used in `start_listening`)
- `TryFrom<AstarteVal> for AstarteData` (FFI → SDK, used in `send`)

### Async bridging pattern

The Rust SDK is fully async. `AstarteDevice` holds a `tokio::Runtime` and uses `rt.block_on(...)` to drive async calls from synchronous FFI entry points. The connection event loop runs as a spawned task inside the same runtime.
