import uniffi.astarte_device_sdk_bindings.*
import kotlin.concurrent.thread

// Define the listener to handle callbacks from Rust
class MyAstarteListener : EventListener {
    override fun onConnected() {
        println("✅ [Kotlin] Device Connected!")
    }

    override fun onDisconnected() {
        println("❌ [Kotlin] Device Disconnected.")
    }

    override fun onDataReceived(interfaceName: String, path: String, data: AstarteVal) {
        // val valueString = when (data) {
        //     is AstarteVal.Integer -> data.value.toString()
        //     is AstarteVal.Double -> data.value.toString()
        //     is AstarteVal.Boolean -> data.value.toString()
        //     is AstarteVal.String -> data.value
        //     is AstarteVal.LongInteger -> data.value.toString()
        //     // Sealed classes are exhaustive, so 'else' is not usually needed
        // }
        // println("📩 [Kotlin] Received: $interfaceName$path = $valueString")
    }
}

fun main() {
    // 1. Create Config
    // REPLACE THESE WITH REAL VALUES
    val config = AstarteConfig(
        realm = "test",
        deviceId = "Pm9iVIl6SUSW4YpSnIArdw",
        credentialsSecret = "2eZm3TBy1q++QYGCdnEZ3quyH265D+rnrFBt88HamOw=",
        pairingUrl = "http://api.astarte.localhost/pairing",
        ignoreSsl = true
    )

    try {
        println("🚀 [Kotlin] Initializing Rust SDK...")

        // Ensure this directory exists relative to where you run gradle
        val device = AstarteDevice(config, "../interfaces")

        thread {
            println("👂 [Kotlin] Starting Event Listener...")
            device.startListening(MyAstarteListener())
            println("👂 [Kotlin] End Event Listener...")
        }

        // Send a test value
        Thread.sleep(1000) // Wait before retrying

        println("📤 [Kotlin] Sending data...")
        while (true) {
            try {
                device.send("org.astarte-platform.python.examples.DeviceDatastream", "/integer_endpoint", AstarteVal.Integer(100))
                println("sent")
            } catch (e: Exception) {
                println("⚠️ Send failed (device might not be connected yet): ${e.message}")
            }
            Thread.sleep(1000) // Wait before retrying
        }
        // Keep alive to receive events
        println("zzz [Kotlin] Waiting for events (Press Ctrl+C to stop)...")
        Thread.sleep(60_000)

    } catch (e: Exception) {
        println("💥 Fatal Error: ${e.message}")
        e.printStackTrace()
    }
}
