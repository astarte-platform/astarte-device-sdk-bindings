# astarte-device-sdk-bindings

Bindings for using the Astarte Device SDK from Java via BoltFFI.

> **Useful Link:** [Getting Started with BoltFFI](https://www.boltffi.dev/docs/getting-started)

---

## Packing the Java Bindings

To generate the Java package using BoltFFI, run:

```sh
boltffi pack java
```

## Building and Running with Gradle (v8)
Follow these steps to build and run the Java SDK using Gradle:

#### 1. Copy Native Libraries Copy the native libraries to the appropriate locations:
``` Bash
cp dist/java/libastarte_device_sdk_bindings_jni.so astarte-sdk-java/app/libs/native
cp dist/java/libastarte_device_sdk_bindings.so astarte-sdk-java/app/target/debug/libastarte_device_sdk_bindings.so
```

#### 2. Build the Java SDK Navigate into the Java SDK directory and build the project:
``` Bash
cd astarte-sdk-java
gradle build
cd ../
```

#### 3. Copy Generated Java Sources Move the generated Java sources to your application’s source directory:
```Bash
cp -r dist/java/org astarte-sdk-java/app/src/main/java/
```

#### 4. Run the Application From the astarte-sdk-java directory, execute:
```Bash
gradle run
```

### Notes
- Ensure all dependencies are installed and your environment is properly set up before building.
- Adapt paths as necessary depending on your project or system structure.
- Refer to the BoltFFI documentation : https://www.boltffi.dev/docs/getting-started for troubleshooting or advanced configurations.
