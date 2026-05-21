package org.astarte.device.sdk.example

import android.os.Bundle
import android.widget.Button
import android.widget.ScrollView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import org.astarte.device.sdk.bindings.AstarteConfig
import org.astarte.device.sdk.bindings.AstarteDevice
import org.astarte.device.sdk.bindings.AstarteVal
import org.astarte.device.sdk.bindings.EventListener
import org.astarte.device.sdk.bindings.ObjectEntry
import kotlin.concurrent.thread

class MainActivity : AppCompatActivity() {

    private lateinit var logText: TextView
    private lateinit var scrollView: ScrollView
    private lateinit var startButton: Button

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        logText = findViewById(R.id.logText)
        scrollView = findViewById(R.id.scrollView)
        startButton = findViewById(R.id.startButton)

        startButton.setOnClickListener {
            startButton.isEnabled = false
            startDemo()
        }
    }

    private fun log(msg: String) {
        android.util.Log.i("AstarteDemo", msg)
        runOnUiThread {
            logText.append("$msg\n")
            scrollView.post { scrollView.fullScroll(ScrollView.FOCUS_DOWN) }
        }
    }

    private fun startDemo() {
        thread {
            // REPLACE THESE WITH REAL VALUES
            val config = AstarteConfig(
                realm = "test",
                deviceId = "Pm9iVIl6SUSW4YpSnIArdw",
                credentialsSecret = "2eZm3TBy1q++QYGCdnEZ3quyH265D+rnrFBt88HamOw=",
                pairingUrl = "http://api.astarte.localhost/pairing",
                ignoreSsl = true
            )

            try {
                log("Initializing Astarte SDK...")
                val device = AstarteDevice(config, "../../interfaces")

                thread {
                    log("Starting event listener...")
                    device.startListening(object : EventListener {
                        override fun onConnected() {
                            log("Connected!")
                        }

                        override fun onDisconnected() {
                            log("Disconnected.")
                        }

                        override fun onDataReceived(`interface`: String, path: String, data: AstarteVal) {
                            log("Data: $`interface`$path = $data")
                        }

                        override fun onObjectReceived(`interface`: String, path: String, entries: List<ObjectEntry>) {
                            log("Object: $`interface`$path (${entries.size} fields)")
                        }

                        override fun onPropertyReceived(`interface`: String, path: String, data: AstarteVal) {
                            log("Property set: $`interface`$path = $data")
                        }

                        override fun onPropertyUnset(`interface`: String, path: String) {
                            log("Property unset: $`interface`$path")
                        }
                    })
                }

                Thread.sleep(1000)

                repeat(10) { i ->
                    try {
                        device.send(
                            "org.astarte-platform.python.examples.DeviceDatastream",
                            "/integer_endpoint",
                            AstarteVal.Integer(i)
                        )
                        log("Sent: $i")
                    } catch (e: Exception) {
                        log("Send failed: ${e.message}")
                    }
                    Thread.sleep(1000)
                }

                device.disconenct()
                device.close()
                log("Done.")

            } catch (e: Exception) {
                log("Error: ${e.message}")
            } finally {
                runOnUiThread { startButton.isEnabled = true }
            }
        }
    }
}
