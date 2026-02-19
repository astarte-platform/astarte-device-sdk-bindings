plugins {
    kotlin("jvm") version "2.3.10"
    application
}

repositories {
    mavenCentral()
}

dependencies {
    // UniFFI requires JNA to load the shared library
    implementation("net.java.dev.jna:jna:5.13.0")

    // Standard Kotlin library
    implementation(kotlin("stdlib"))

    // Coroutines (optional, but good if you expand the app)
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")

}

application {
    // Points to the 'main' function in Main.kt
    mainClass.set("MainKt")
}

kotlin {
    jvmToolchain(17)
}

tasks.named<JavaExec>("run") {
    // ---------------------------------------------------------
    // IMPORTANT: Point this to your Rust project's release folder
    // ---------------------------------------------------------
    val rustLibPath = file("../target/release")
    val rustLibPathR = file("../target/release/libastarte_device_sdk_bindings.so")

    // Verify the path exists to help debug
    doFirst {
        if (!rustLibPath.exists()) {
            throw GradleException("Could not find Rust library at: $rustLibPath. Please build the Rust crate first.")
        }
        if (!rustLibPathR.exists()) {
            throw GradleException("Could not find Rust library at: $rustLibPath. Please build the Rust crate first.")
        }
        println("🔗 Linking Rust library from: $rustLibPath $rustLibPathR")
    }

    // Tell Java where to look for libastarte_bindings.so/dylib/dll
    systemProperty("java.library.path", rustLibPath)
}
