package org.astarte.device.sdk.bindings;

import java.util.concurrent.atomic.AtomicBoolean;

/**
 * The main entry point for communicating with the Astarte platform.
 *
 * <p>{@code AstarteDevice} wraps the Rust Astarte Device SDK over JNI. It holds a native
 * resource and therefore implements {@link AutoCloseable} — always use it inside a
 * {@code try-with-resources} block or call {@link #close()} explicitly.
 *
 * <p>Typical usage:
 * <pre>{@code
 * AstarteConfig cfg = new AstarteConfig("realm", "deviceId", "secret",
 *         "http://api.astarte.example/pairing", false);
 * try (AstarteDevice device = new AstarteDevice(cfg, "/path/to/interfaces")) {
 *     ExecutorService executor = Executors.newSingleThreadExecutor();
 *     executor.submit(() -> device.startListening(myListener));
 *     device.send("com.example.Sensor", "/temperature", new AstarteVal.Double(22.5));
 *     // ...
 * }
 * }</pre>
 */
public final class AstarteDevice implements AutoCloseable {
    private final long handle;
    private final AtomicBoolean closed = new AtomicBoolean(false);

    private AstarteDevice(long handle) {
        this.handle = handle;
    }

    /**
     * Creates a new {@code AstarteDevice} and connects to the Astarte broker.
     *
     * @param config        MQTT credentials and pairing endpoint
     * @param interfacesDir path to the directory containing Astarte interface JSON files
     * @throws SdkError.Exception if the configuration is invalid or the connection fails
     */
    public AstarteDevice(AstarteConfig config, String interfacesDir) {
        this(AstarteDevice.createHandle0(config, interfacesDir));
    }

    private static long createHandle0(AstarteConfig config, String interfacesDir) {
        try (
            WireWriter _wire_config = new WireWriter(config.wireEncodedSize())
        ) {
            config.wireEncodeTo(_wire_config);
            long _handle = Native.boltffi_astarte_device_new(_wire_config.toBuffer(), interfacesDir.getBytes(java.nio.charset.StandardCharsets.UTF_8));
            if (_handle == 0L) throw new RuntimeException("Constructor failed");
            return _handle;
        }
    }

    long rawHandle() {
        return handle;
    }

    /**
     * Releases the native device handle. Idempotent — safe to call more than once.
     */
    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) return;
        Native.boltffi_astarte_device_free(handle);
    }

    /**
     * Publishes an individual datastream value on the given interface and path.
     *
     * <p>The call is synchronous from the Java side; the underlying Rust runtime drives
     * the async send to completion before returning.
     *
     * @param interfaceName fully qualified Astarte interface name (e.g. {@code "com.example.Sensor"})
     * @param interfacePath endpoint path on that interface (e.g. {@code "/temperature"})
     * @param data          the value to publish, wrapped in the appropriate {@link AstarteVal} variant
     * @throws SdkError.Exception if the send fails (e.g. not connected, invalid interface/path)
     */
    public void send(String interfaceName, String interfacePath, AstarteVal data) {
        try (
            WireWriter _wire_data = new WireWriter(data.wireEncodedSize())
        ) {
            data.wireEncodeTo(_wire_data);
            Native.boltffi_astarte_device_send(handle, interfaceName.getBytes(java.nio.charset.StandardCharsets.UTF_8), interfacePath.getBytes(java.nio.charset.StandardCharsets.UTF_8), _wire_data.toBuffer());
        }
    }

    /**
     * Starts the event receive loop, dispatching incoming Astarte events to {@code listener}.
     *
     * <p><strong>This method blocks</strong> until the device disconnects or an error occurs.
     * Call it on a dedicated background thread (e.g. via an {@code ExecutorService}).
     *
     * @param listener callback implementation that receives connect, disconnect, and data events
     */
    public void startListening(EventListener listener) {
        Native.boltffi_astarte_device_start_listening(handle, EventListenerCallbacks.create(listener));
    }

    /**
     * Disconnects the device from the Astarte broker.
     *
     * <p>After this call {@link #startListening} will return on the background thread.
     * The device handle remains valid until {@link #close()} is called.
     *
     * @throws SdkError.Exception if the disconnect fails
     */
    public void disconenct() {
        Native.boltffi_astarte_device_disconenct(handle);
    }
}