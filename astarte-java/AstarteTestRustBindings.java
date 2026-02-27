
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

public class AstarteTestRustBindings {

    static MethodHandle addNumbers; // Wrapper for the Rust function
    static MethodHandle newAstarteDevice; // Wrapper for the Rust function

    static {
        // Initialize the linker
        Linker linker = Linker.nativeLinker();

        // Load the Rust library
        SymbolLookup lib = SymbolLookup.libraryLookup("../target/release/libastarte_device_sdk_bindings.so", Arena.global());

        // StructLayout astarteDevice = MemoryLayout.structLayout(
        //         ValueLayout.JAVA_INT.withName("x") // Field `x` (i32 in Rust)
        // );

        newAstarteDevice = linker.downcallHandle(
                lib.find("new").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_INT)
        );

        // var arena = Arena.ofConfined();  // Confined Arena for memory management

        // // Allocate memory for the struct
        // MemorySegment astarteDevSegment = arena.allocate(astarteDevice);

        // Link the Rust function
        addNumbers = linker.downcallHandle(
                lib.find("add_numbers").orElseThrow(), // Replace with the function name from Rust
                FunctionDescriptor.of(
                        ValueLayout.JAVA_INT, // Rust's return type: i32
                        ValueLayout.ADDRESS, // Rust's first parameter: i32
                        ValueLayout.JAVA_INT // Rust's second parameter: i32
                )
        );
    }
}
