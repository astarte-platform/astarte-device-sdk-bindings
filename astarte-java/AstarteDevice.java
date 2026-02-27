
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

public class AstarteDevice implements AutoCloseable {

    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;

    // Method Handles
    private static final MethodHandle NEW_HANDLE;
    private static final MethodHandle SEND_HANDLE;
    private static final MethodHandle LISTEN_HANDLE;
    private static final MethodHandle FREE_HANDLE;

    static {
        // Load the shared library compiled from Rust (e.g., libastarte_wrapper.so)
        LOOKUP = SymbolLookup.libraryLookup("../target/release/libastarte_device_sdk_bindings.so", Arena.global());

        NEW_HANDLE = LINKER.downcallHandle(
                LOOKUP.find("astarte_device_new").orElseThrow(),
                FunctionDescriptor.of(
                        ValueLayout.ADDRESS, // Return: *mut AstarteDevice
                        ValueLayout.ADDRESS, // realm
                        ValueLayout.ADDRESS, // device_id
                        ValueLayout.ADDRESS, // credentials_secret
                        ValueLayout.ADDRESS, // pairing_url
                        ValueLayout.JAVA_BOOLEAN,// ignore_ssl
                        ValueLayout.ADDRESS // interfaces_dir
                )
        );

        SEND_HANDLE = LINKER.downcallHandle(
                LOOKUP.find("astarte_device_send").orElseThrow(),
                FunctionDescriptor.ofVoid(
                        ValueLayout.ADDRESS, // ptr
                        ValueLayout.ADDRESS, // interface_name
                        ValueLayout.ADDRESS, // interface_path
                        ValueLayout.JAVA_INT // value
                )
        );

        LISTEN_HANDLE = LINKER.downcallHandle(
                LOOKUP.find("astarte_device_start_listening").orElseThrow(),
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );

        FREE_HANDLE = LINKER.downcallHandle(
                LOOKUP.find("astarte_device_free").orElseThrow(),
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );
    }

    // The raw pointer to the Rust struct
    private MemorySegment devicePointer;
    // Arena attached to the lifecycle of this Java object
    private final Arena objectArena;

    public AstarteDevice(String realm, String deviceId, String secret,
            String pairingUrl, boolean ignoreSsl, String interfacesDir) {

        this.objectArena = Arena.ofConfined();

        try {
            this.devicePointer = (MemorySegment) NEW_HANDLE.invokeExact(
                    objectArena.allocateFrom(realm),
                    objectArena.allocateFrom(deviceId),
                    objectArena.allocateFrom(secret),
                    objectArena.allocateFrom(pairingUrl),
                    ignoreSsl,
                    objectArena.allocateFrom(interfacesDir)
            );

            if (this.devicePointer.equals(MemorySegment.NULL)) {
                throw new RuntimeException("Failed to initialize AstarteDevice in Rust");
            }
        } catch (Throwable e) {
            objectArena.close();
            throw new RuntimeException(e);
        }
    }

    public void send(String interfaceName, String interfacePath, int value) {
        // Use a short-lived auto arena for method arguments
        try (Arena methodArena = Arena.ofConfined()) {
            SEND_HANDLE.invokeExact(
                    this.devicePointer,
                    methodArena.allocateFrom(interfaceName),
                    methodArena.allocateFrom(interfacePath),
                    value
            );
        } catch (Throwable e) {
            throw new RuntimeException("Failed to send data", e);
        }
    }

    public void startListening() {
        try {
            LISTEN_HANDLE.invokeExact(this.devicePointer);
        } catch (Throwable e) {
            throw new RuntimeException("Error while listening", e);
        }
    }

    @Override
    public void close() {
        if (this.devicePointer != null && !this.devicePointer.equals(MemorySegment.NULL)) {
            try {
                // Free the Rust memory
                FREE_HANDLE.invokeExact(this.devicePointer);
                this.devicePointer = MemorySegment.NULL;
            } catch (Throwable e) {
                System.err.println("Failed to free Rust memory: " + e.getMessage());
            }
        }
        // Close the Java arena, freeing the C-strings allocated in the constructor
        if (this.objectArena != null) {
            this.objectArena.close();
        }
    }
}
