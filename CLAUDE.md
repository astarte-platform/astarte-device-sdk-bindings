# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Does

This project exposes the [Astarte Device SDK (Rust)](https://github.com/astarte-platform/astarte-device-sdk-rust) to multiple platforms via a native FFI bridge built with [BoltFFI](https://www.boltffi.dev/docs/getting-started):

- **Java** (JVM, Gradle 8) — JNI glue + generated Java wrappers
- **Android** (Kotlin, AGP 4.2.2, min SDK 24) — native `.so` per ABI + generated Kotlin source
- **C#** (.NET 8, Linux) — generated C# source via P/Invoke

The Rust crate compiles to a shared library; BoltFFI generates the language-specific glue and wrapper classes automatically.

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

### Generate Android bindings

`get_android` sources `$HOME/Android/Sdk/export.sh` (sets `ANDROID_HOME`, `ANDROID_NDK`, and `PATH`).

```sh
get_android
export JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64"
boltffi pack android    # compiles Rust for Android ABIs, generates Kotlin source and jniLibs
```

Output lands in `dist/android/`:
- `jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}/libastarte-device-sdk-bindings.so`
- `kotlin/org/astarte/device/sdk/bindings/AstarteDeviceSdkBindings.kt` — all types in one file

**Post-generation fixes** required in `dist/android/kotlin/…/AstarteDeviceSdkBindings.kt` (apply these once, then copy):

1. **Missing `kotlinx.coroutines` imports** — add after the existing `kotlinx.coroutines.*` imports:
   ```kotlin
   import kotlinx.coroutines.CoroutineScope
   import kotlinx.coroutines.Dispatchers
   import kotlinx.coroutines.Job
   import kotlinx.coroutines.SupervisorJob
   import kotlinx.coroutines.launch
   ```

2. **Shadowed stdlib types in `AstarteVal`** — qualify four data class property types:
   ```kotlin
   data class Double(val `value`: kotlin.Double) : AstarteVal()
   data class Boolean(val `value`: kotlin.Boolean) : AstarteVal()
   data class DoubleArray(val `value`: kotlin.DoubleArray) : AstarteVal()
   data class BooleanArray(val `value`: kotlin.BooleanArray) : AstarteVal()
   ```

3. **`interface` keyword as parameter name** — escape with backticks in the `Native` object's `external fun` declarations:
   ```kotlin
   // change: , interface: ByteBuffer,
   // to:     , `interface`: ByteBuffer,
   ```

4. **`run {}` in secondary constructor** — qualify the inner `run` block to avoid `this`-before-init error:
   ```kotlin
   // In AstarteDevice constructor, change inner bare `run {` to `kotlin.run {`
   ```

### Wire up and run the Android app (Gradle 7 / AGP 4.2.2)

```sh
# Copy .so files (arm64-v8a and x86_64 recommended)
cp -r dist/android/jniLibs/arm64-v8a  astarte-sdk-android/app/src/main/jniLibs/
cp -r dist/android/jniLibs/x86_64     astarte-sdk-android/app/src/main/jniLibs/

# Apply post-generation fixes to the Kotlin source (see above), then copy
cp dist/android/kotlin/org/astarte/device/sdk/bindings/AstarteDeviceSdkBindings.kt \
   astarte-sdk-android/app/src/main/kotlin/org/astarte/device/sdk/bindings/

export JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64"
export ANDROID_HOME=$HOME/Android/Sdk
cd astarte-sdk-android

./gradlew test          # run local JVM unit tests (no device needed)
./gradlew assembleDebug # build APK
./gradlew installDebug  # deploy to connected device/emulator
```

The `.so` loads via `System.loadLibrary("astarte-device-sdk-bindings")` (detected automatically on Android by the `Native` object in the generated file).

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
dist/android/               ← BoltFFI-generated artifacts (jniLibs + Kotlin source)
dist/csharp/                ← BoltFFI-generated artifacts (C# source + NuGet layout)
astarte-sdk-java/           ← Gradle multi-project: app / list / utilities
  app/src/main/java/org/
    astarte/device/sdk/bindings/   ← copied from dist/java after boltffi pack
    example/app/App.java           ← demo entry point
astarte-sdk-android/        ← Android Gradle project (AGP 4.2.2, min SDK 24)
  app/src/main/
    kotlin/org/astarte/device/sdk/
      bindings/             ← copied from dist/android/kotlin/ (with post-gen fixes)
      example/MainActivity.kt  ← demo entry point
    jniLibs/{arm64-v8a,x86_64}/   ← copied from dist/android/jniLibs/
astarte-sdk-csharp/         ← .NET 8 solution (library + tests + example)
  AstarteSdk/               ← library project (copied from dist/csharp/src/ + post-gen fix)
  AstarteSdk.Tests/         ← xUnit tests (no native library needed)
  AstarteSdkExample/        ← console demo app
```

### BoltFFI macro conventions in `src/lib.rs`

| Macro / attribute | Purpose |
|---|---|
| `#[data]` | Marks a `struct` or `enum` as a plain data type — generates a Java POJO, Kotlin data class, or C# record |
| `#[boltffi::error]` | Marks an `enum` as the error type — surfaces as `SdkError` (Java exception / C# exception) |
| `#[export]` on a `trait` | Generates a callback interface that the caller implements (`EventListener`) |
| `#[export]` on an `impl` block | Generates a wrapper class around the Rust object with matching methods (`AstarteDevice`) |

### Key types

- `AstarteConfig` — plain data struct holding MQTT credentials and pairing URL; generated as a Java POJO, Kotlin data class, or C# `readonly record struct`
- `AstarteDevice` — the main object; wraps `DeviceClient<Mqtt<MemoryStore>>` and a `tokio::Runtime` so blocking FFI calls can drive async SDK operations
- `AstarteVal` — FFI-safe enum covering all 13 representable `AstarteData` variants (see table below); `BinaryBlobArray` is omitted because nested `Vec<Vec<u8>>` is unsupported by BoltFFI's `#[data]` macro
- `ObjectEntry` — plain data struct (`key: String`, `value: AstarteVal`) used to represent one field of an Object-aggregation event; passed as `Vec<ObjectEntry>` in `EventListener::on_object_received`
- `EventListener` — callback interface implemented by callers to receive connect/disconnect/data events; has six methods: `on_connected`, `on_disconnected`, `on_data_received`, `on_object_received`, `on_property_received`, `on_property_unset`
- `SdkError` — FFI error enum surfaced as an exception in all target languages

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

---

## Generate C# bindings

BoltFFI v0.25+ supports C# code generation via `boltffi pack csharp`. Generated sources land in `dist/csharp/src/` and are then copied to `astarte-sdk-csharp/AstarteSdk/` with post-generation fixes applied, mirroring the Java/Android workflow.

### Generate and pack

```sh
boltffi pack csharp --release   # compiles Rust for linux-x64, generates C# sources + NuGet layout
```

Output lands in `dist/csharp/`:
- `src/*.cs` — generated C# sources (namespace `AstarteDeviceSdkBindings`)
- `runtimes/linux-x64/native/libastarte_device_sdk_bindings.so` — native library
- `BoltFFI.CSharp.csproj` — reference project (targets net10.0; the app project uses net8.0)

The `[targets.csharp]` section in `boltffi.toml`:

```toml
[targets.csharp]
enabled = true
output = "dist/csharp"

[targets.csharp.nuget]
package_id = "Astarte.Device.Sdk.Bindings"
version = "0.1.0"
authors = ["Secomind"]
```

### Wire up the C# app (project layout)

After `boltffi pack csharp`, copy the generated files and apply the post-generation fix:

```sh
cp dist/csharp/src/*.cs astarte-sdk-csharp/AstarteSdk/
# then apply the post-generation fix described below
```

```
astarte-sdk-csharp/
├── AstarteSdk.sln
├── AstarteSdk/                           ← library (net8.0, AllowUnsafeBlocks)
│   ├── AstarteDeviceSdkBindings.cs       ← generated: WireReader/Writer, vtable, NativeMethods
│   ├── AstarteConfig.cs                  ← generated
│   ├── AstarteVal.cs                     ← generated (abstract record, 13 sealed record variants)
│   ├── ObjectEntry.cs                    ← generated (readonly record struct)
│   ├── SdkError.cs                       ← generated (abstract record + SdkErrorException)
│   └── AstarteDevice.cs                  ← generated + post-generation fix
├── AstarteSdk.Tests/                     ← xUnit tests (no native library needed)
│   ├── AstarteValTests.cs
│   ├── AstarteConfigTests.cs
│   ├── ObjectEntryTests.cs
│   └── WireCodecTests.cs                 ← encode/decode round-trips
└── AstarteSdkExample/                    ← console demo app
    └── Program.cs
```

### Post-generation fix: missing `AstarteDevice` constructor

The C# generator does not emit the `new` constructor for `AstarteDevice` (boltffi issue). Apply this fix to the **copied** `AstarteDevice.cs` (not the `dist/` original):

**1. In `AstarteDeviceSdkBindings.cs`, inside `NativeMethods`, add before `AstarteDeviceFree`:**

```csharp
[DllImport(LibName, EntryPoint = "boltffi_astarte_device_new")]
internal static extern IntPtr AstarteDeviceNew(byte[] config, UIntPtr configLen, byte[] interfacesDir, UIntPtr interfacesDirLen);
```

**2. In `AstarteDevice.cs`, add after the private `AstarteDevice(IntPtr handle)` constructor:**

```csharp
using System.Runtime.InteropServices;  // add to top-level usings

public AstarteDevice(AstarteConfig config, string interfacesDir)
    : this(CreateHandle(config, interfacesDir)) { }

private static IntPtr CreateHandle(AstarteConfig config, string interfacesDir)
{
    using var configWire = new WireWriter(config.WireEncodedSize());
    config.WireEncodeTo(configWire);
    byte[] configBytes = configWire.ToArray();
    byte[] dirBytes = Encoding.UTF8.GetBytes(interfacesDir);
    IntPtr handle = NativeMethods.AstarteDeviceNew(
        configBytes, (UIntPtr)configBytes.Length,
        dirBytes, (UIntPtr)dirBytes.Length);
    if (handle == IntPtr.Zero)
        throw new SdkErrorException(new SdkError.Config("Failed to create AstarteDevice"));
    return handle;
}
```

### Array equality note (C# records)

`AstarteVal` variants that hold arrays (`BinaryBlob`, `DoubleArray`, etc.) are `sealed record` types. C# records use reference equality for array fields, so two instances with the same content are **not** record-equal. Compare `.Value` directly (xUnit's `Assert.Equal` on arrays does structural comparison).

### Copy and build (Linux, .NET 8)

```sh
# 1. Copy the native shared library next to the example binary
cp dist/csharp/runtimes/linux-x64/native/libastarte_device_sdk_bindings.so \
   astarte-sdk-csharp/AstarteSdkExample/

# 2. Build everything
cd astarte-sdk-csharp
dotnet build

# 3. Run unit tests (no native library needed – tests are pure managed code)
dotnet test AstarteSdk.Tests/

# 4. Run the example (requires a live Astarte broker and real credentials)
dotnet run --project AstarteSdkExample/
```

On Linux the runtime searches for `libastarte_device_sdk_bindings.so` next to the executable, then `LD_LIBRARY_PATH`, then the standard library search path.
