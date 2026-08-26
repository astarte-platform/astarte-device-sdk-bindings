package org.astarte.device;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Optional;

import static org.astarte.device.internal.Layouts.*;

/**
 * Loads the astarte_device_sdk_bindings shared library and provides MethodHandles
 * for all exported C functions.
 */
public final class NativeLoader {
    private NativeLoader() {}

    private static final SymbolLookup LOOKUP;
    public static final Linker LINKER = Linker.nativeLinker();

    // TODO just check the library and the env do not search in local paths
    static {
        // Load the shared library.
        // Try ASTARTE_SDK_LIB_PATH first, then debug/release targets.
        String envPath = System.getenv("ASTARTE_SDK_LIB_PATH");
        Path libPath = null;
        if (envPath != null && Files.exists(Paths.get(envPath))) {
            libPath = Paths.get(envPath);
        } else {
            String osName = System.getProperty("os.name").toLowerCase();
            String libName = "libastarte_device_sdk_bindings.so";
            if (osName.contains("mac")) {
                libName = "libastarte_device_sdk_bindings.dylib";
            } else if (osName.contains("win")) {
                libName = "astarte_device_sdk_bindings.dll";
            }
            
            // Assume we are running from somewhere within the project.
            // A more robust way would be needed for production, but this matches Python's logic.
            Path current = Paths.get("").toAbsolutePath();
            // Try to find the target directory in current or parent directories
            while (current != null) {
                Path debug = current.resolve("target/debug/" + libName);
                if (Files.exists(debug)) {
                    libPath = debug;
                    break;
                }
                Path release = current.resolve("target/release/" + libName);
                if (Files.exists(release)) {
                    libPath = release;
                    break;
                }
                current = current.getParent();
            }
            if (libPath == null) {
                // Fallback to expecting it in java.library.path
                try {
                    System.loadLibrary("astarte_device_sdk_bindings");
                } catch (UnsatisfiedLinkError e) {
                    throw new RuntimeException("Could not find " + libName + ". Please build the Rust project or set ASTARTE_SDK_LIB_PATH.");
                }
            }
        }
        
        if (libPath != null) {
            // NOTE is the global arena a good idea ? what if this class get unloaded
            // maybe we should implement AutoClosable and have a local arena ?
            LOOKUP = SymbolLookup.libraryLookup(libPath, Arena.global());
        } else {
            LOOKUP = SymbolLookup.loaderLookup();
        }
    }

    private static MethodHandle downcall(String name, FunctionDescriptor desc) {
        return LOOKUP.find(name)
                .map(addr -> LINKER.downcallHandle(addr, desc))
                .orElseThrow(() -> new RuntimeException("Missing symbol: " + name));
    }

    // NativeDeviceHandle device_handle_init(void);
    public static final MethodHandle DEVICE_HANDLE_INIT = downcall("device_handle_init",
            FunctionDescriptor.of(PTR)
    );

    // void device_handle_connect(NativeDeviceHandle handle, const struct NativeDeviceConfig *config, DeviceHandleBuildCallback build_cbk, UserData build_user_data, DeviceHandleLoopCallback loop_cbk, UserData loop_user_data);
    public static final MethodHandle DEVICE_HANDLE_CONNECT = downcall("device_handle_connect",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR, PTR, PTR)
    );

    // void device_handle_receive(NativeDeviceHandle device_handle, DeviceHandleReceiveCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_RECEIVE = downcall("device_handle_receive",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR)
    );

    // void device_handle_free_device_event(struct NativeDeviceEvent *event);
    public static final MethodHandle DEVICE_HANDLE_FREE_DEVICE_EVENT = downcall("device_handle_free_device_event",
            FunctionDescriptor.ofVoid(PTR)
    );

    // void device_handle_send_individual(NativeDeviceHandle device_handle, const struct NativeIndividualSend *data, DeviceHandleSendCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_SEND_INDIVIDUAL = downcall("device_handle_send_individual",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR)
    );

    // void device_handle_send_object(NativeDeviceHandle device_handle, const struct NativeObjectSend *data, DeviceHandleSendCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_SEND_OBJECT = downcall("device_handle_send_object",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR)
    );

    // void device_handle_set_property(NativeDeviceHandle device_handle, const struct NativeSetProperty *property, DeviceHandleSendCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_SET_PROPERTY = downcall("device_handle_set_property",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR)
    );

    // void device_handle_unset_property(NativeDeviceHandle device_handle, const struct NativePropertyIdentifier *property, DeviceHandleSendCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_UNSET_PROPERTY = downcall("device_handle_unset_property",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR)
    );

    // void device_handle_get_property(NativeDeviceHandle device_handle, const struct NativePropertyIdentifier *property, DeviceHandleGetCallback callback, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_GET_PROPERTY = downcall("device_handle_get_property",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR, PTR)
    );

    // void device_handle_free_get_property(struct NativeOption_NativeDeviceData *property);
    public static final MethodHandle DEVICE_HANDLE_FREE_GET_PROPERTY = downcall("device_handle_free_get_property",
            FunctionDescriptor.ofVoid(PTR)
    );

    // void device_handle_disconnect(NativeDeviceHandle handle, DeviceHandleDisconnectCallback disconnect_cbk, UserData user_data);
    public static final MethodHandle DEVICE_HANDLE_DISCONNECT = downcall("device_handle_disconnect",
            FunctionDescriptor.ofVoid(PTR, PTR, PTR)
    );

    // void device_handle_free(NativeDeviceHandle handle);
    public static final MethodHandle DEVICE_HANDLE_FREE = downcall("device_handle_free",
            FunctionDescriptor.ofVoid(PTR)
    );
    
    // Upcall descriptors
    // FIXME here we probably need pointers with a target layout 
    public static final FunctionDescriptor CALLBACK_BOOL_DESC = FunctionDescriptor.ofVoid(NATIVE_STRING_RESULT_BOOL_PTR, PTR);
    public static final FunctionDescriptor CALLBACK_EVENT_DESC = FunctionDescriptor.ofVoid(NATIVE_STRING_RESULT_EVENT_PTR, PTR);
    public static final FunctionDescriptor CALLBACK_GET_PROP_DESC = FunctionDescriptor.ofVoid(NATIVE_STRING_RESULT_OPTIONAL_DEVICE_DATA_PTR, PTR);
}
