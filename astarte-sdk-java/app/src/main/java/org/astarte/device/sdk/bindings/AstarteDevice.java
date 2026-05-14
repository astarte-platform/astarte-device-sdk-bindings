package org.astarte.device.sdk.bindings;

import java.util.concurrent.atomic.AtomicBoolean;

public final class AstarteDevice implements AutoCloseable {
    private final long handle;
    private final AtomicBoolean closed = new AtomicBoolean(false);

    private AstarteDevice(long handle) {
        this.handle = handle;
    }

    /**
     * Constructor: Builds the SDK instance
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

    @Override
    public void close() {
        if (!closed.compareAndSet(false, true)) return;
        Native.boltffi_astarte_device_free(handle);
    }

    public void send(String interfaceName, String interfacePath, AstarteVal data) {
        try (
            WireWriter _wire_data = new WireWriter(data.wireEncodedSize())
        ) {
            data.wireEncodeTo(_wire_data);
            Native.boltffi_astarte_device_send(handle, interfaceName.getBytes(java.nio.charset.StandardCharsets.UTF_8), interfacePath.getBytes(java.nio.charset.StandardCharsets.UTF_8), _wire_data.toBuffer());
        }
    }

    public void startListening(EventListener listener) {
        Native.boltffi_astarte_device_start_listening(handle, EventListenerCallbacks.create(listener));
    }

    public void disconenct() {
        Native.boltffi_astarte_device_disconenct(handle);
    }
}
