
public class Main {

    void main() throws Throwable {
        // Call the Rust function
        // MemorySegment device = (MemorySegment) AstarteRustBindings.newAstarteDevice.invoke(40);
        // int result = (int) AstarteRustBindings.addNumbers.invokeExact(device, 80);
        // IO.println("Result: " + result); // Output should be 30

        IO.println(" START ");

        try (AstarteDevice device = new AstarteDevice(
                "test",
                "Pm9iVIl6SUSW4YpSnIArdw",
                "2eZm3TBy1q++QYGCdnEZ3quyH265D+rnrFBt88HamOw=",
                "http://api.astarte.localhost/pairing",
                true,
                "../interfaces")) {

            Thread.ofVirtual()
                    .name("astarte-listener-thread")
                    .start(() -> {
                        IO.println("Starting listener on: " + Thread.currentThread());
                        device.startListening();
                    });

            IO.println("Main thread is free. Simulating other work...");

            for (int i = 0; i < 5; i++) {
                Thread.sleep(1000);
                // We can even send more data concurrently!
                device.send("org.astarte-platform.python.examples.DeviceDatastream", "/integer_endpoint", 42+i);
            }
        }
        // Rust memory is safely freed here!
        IO.println(" STOP ");
    }
}
