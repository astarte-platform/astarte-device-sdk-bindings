package org.astarte.device.sdk.bindings;
final class EventListenerCallbacks {
    private static final java.util.concurrent.ConcurrentHashMap<Long, EventListener> HANDLE_MAP = new java.util.concurrent.ConcurrentHashMap<>();
    private static final java.util.concurrent.atomic.AtomicLong COUNTER = new java.util.concurrent.atomic.AtomicLong(1L);

    private EventListenerCallbacks() {}

    private static long insert(EventListener impl_) {
        long handle = COUNTER.getAndAdd(2L);
        HANDLE_MAP.put(handle, impl_);
        return handle;
    }

    public static void free(long handle) {
        HANDLE_MAP.remove(handle);
    }

    public static long clone(long handle) {
        EventListener obj = HANDLE_MAP.get(handle);
        return obj != null ? insert(obj) : 0L;
    }

    private static final class Proxy implements EventListener {
        private final long handle;

        private Proxy(long handle) {
            this.handle = handle;
        }

        long rawHandle() {
            return handle;
        }
        @Override
        protected void finalize() throws Throwable {
            try {
                if (handle != 0L) {
                    Native.boltffiCallbackEventListenerRelease(handle);
                }
            } finally {
                super.finalize();
            }
        }

        @Override
        public void onConnected() {
            Native.boltffiCallbackEventListenerOnConnected(handle);
        }

        @Override
        public void onDisconnected() {
            Native.boltffiCallbackEventListenerOnDisconnected(handle);
        }

        @Override
        public void onDataReceived(String _interface, String path, AstarteVal data) {
            try (
                WireWriter _wire_interface = new WireWriter((4 + (4 + (_interface).length() * 3)));
                WireWriter _wire_path = new WireWriter((4 + (4 + (path).length() * 3)));
                WireWriter _wire_data = new WireWriter(data.wireEncodedSize())
            ) {
                _wire_interface.writeString(_interface);
                _wire_path.writeString(path);
                data.wireEncodeTo(_wire_data);
            Native.boltffiCallbackEventListenerOnDataReceived(handle, _interface.getBytes(java.nio.charset.StandardCharsets.UTF_8), path.getBytes(java.nio.charset.StandardCharsets.UTF_8), _wire_data.toBuffer());
            }
        }

        @Override
        public void onObjectReceived(String _interface, String path, java.util.List<ObjectEntry> entries) {
            try (
                WireWriter _wire_interface = new WireWriter((4 + (4 + (_interface).length() * 3)));
                WireWriter _wire_path = new WireWriter((4 + (4 + (path).length() * 3)));
                WireWriter _wire_entries = new WireWriter(WireWriter.listWireSize(entries, sizeItem0 -> sizeItem0.wireEncodedSize()))
            ) {
                _wire_interface.writeString(_interface);
                _wire_path.writeString(path);
                _wire_entries.writeI32(entries.size()); for (ObjectEntry item0 : entries) { item0.wireEncodeTo(_wire_entries); };
            Native.boltffiCallbackEventListenerOnObjectReceived(handle, _interface.getBytes(java.nio.charset.StandardCharsets.UTF_8), path.getBytes(java.nio.charset.StandardCharsets.UTF_8), _wire_entries.toBuffer());
            }
        }

        @Override
        public void onPropertyReceived(String _interface, String path, AstarteVal data) {
            try (
                WireWriter _wire_interface = new WireWriter((4 + (4 + (_interface).length() * 3)));
                WireWriter _wire_path = new WireWriter((4 + (4 + (path).length() * 3)));
                WireWriter _wire_data = new WireWriter(data.wireEncodedSize())
            ) {
                _wire_interface.writeString(_interface);
                _wire_path.writeString(path);
                data.wireEncodeTo(_wire_data);
            Native.boltffiCallbackEventListenerOnPropertyReceived(handle, _interface.getBytes(java.nio.charset.StandardCharsets.UTF_8), path.getBytes(java.nio.charset.StandardCharsets.UTF_8), _wire_data.toBuffer());
            }
        }

        @Override
        public void onPropertyUnset(String _interface, String path) {
            try (
                WireWriter _wire_interface = new WireWriter((4 + (4 + (_interface).length() * 3)));
                WireWriter _wire_path = new WireWriter((4 + (4 + (path).length() * 3)))
            ) {
                _wire_interface.writeString(_interface);
                _wire_path.writeString(path);
            Native.boltffiCallbackEventListenerOnPropertyUnset(handle, _interface.getBytes(java.nio.charset.StandardCharsets.UTF_8), path.getBytes(java.nio.charset.StandardCharsets.UTF_8));
            }
        }
    }

    static EventListener wrap(long handle) {
        EventListener existing = HANDLE_MAP.get(handle);
        return existing != null ? existing : new Proxy(handle);
    }

    public static void on_connected(long handle) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onConnected();
    }

    public static void on_disconnected(long handle) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onDisconnected();
    }

    public static void on_data_received(long handle, java.nio.ByteBuffer _interface, java.nio.ByteBuffer path, java.nio.ByteBuffer data) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onDataReceived(WireReader.decodeBuffer(_interface, reader -> reader.readString()), WireReader.decodeBuffer(path, reader -> reader.readString()), WireReader.decodeBuffer(data, reader -> AstarteVal.decode(reader)));
    }

    public static void on_object_received(long handle, java.nio.ByteBuffer _interface, java.nio.ByteBuffer path, java.nio.ByteBuffer entries) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onObjectReceived(WireReader.decodeBuffer(_interface, reader -> reader.readString()), WireReader.decodeBuffer(path, reader -> reader.readString()), WireReader.decodeBuffer(entries, reader -> reader.readList(readIndex0 -> ObjectEntry.decode(reader))));
    }

    public static void on_property_received(long handle, java.nio.ByteBuffer _interface, java.nio.ByteBuffer path, java.nio.ByteBuffer data) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onPropertyReceived(WireReader.decodeBuffer(_interface, reader -> reader.readString()), WireReader.decodeBuffer(path, reader -> reader.readString()), WireReader.decodeBuffer(data, reader -> AstarteVal.decode(reader)));
    }

    public static void on_property_unset(long handle, java.nio.ByteBuffer _interface, java.nio.ByteBuffer path) {
        EventListener impl_ = HANDLE_MAP.get(handle);
        if (impl_ == null) throw new RuntimeException("invalid callback handle");
        impl_.onPropertyUnset(WireReader.decodeBuffer(_interface, reader -> reader.readString()), WireReader.decodeBuffer(path, reader -> reader.readString()));
    }

    static long create(EventListener impl_) {
        return insert(impl_);
    }
}