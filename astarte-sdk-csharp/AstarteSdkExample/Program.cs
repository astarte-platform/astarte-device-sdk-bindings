// Astarte Device SDK – C# example
// Mirrors the Java App.java and Android MainActivity.kt demos.
//
// Prerequisites:
//   1. Copy libastarte_device_sdk_bindings.so next to this executable, or set LD_LIBRARY_PATH.
//   2. Replace the placeholder credentials below with real Astarte realm/device values.
//   3. Ensure the interfaces/ directory path is reachable.

using System;
using System.Threading;
using AstarteDeviceSdkBindings;

var config = new AstarteConfig(
    Realm:             "test",
    DeviceId:          "Pm9iVIl6SUSW4YpSnIArdw",
    CredentialsSecret: "2eZm3TBy1q++QYGCdnEZ3quyH265D+rnrFBt88HamOw=",
    PairingUrl:        "http://api.astarte.localhost/pairing",
    IgnoreSsl:         true);

Console.WriteLine("Initializing Astarte SDK...");

using var device = new AstarteDevice(config, "../../interfaces");

// Start the event loop on a background thread (StartListening blocks until disconnected)
var listenerThread = new Thread(() =>
{
    Console.WriteLine("Starting event listener...");
    device.StartListening(new ConsoleEventListener());
}) { IsBackground = true };
listenerThread.Start();

Thread.Sleep(1000); // give the connection a moment to establish

for (int i = 0; i < 10; i++)
{
    try
    {
        device.Send(
            "org.astarte-platform.python.examples.DeviceDatastream",
            "/integer_endpoint",
            new AstarteVal.Integer(i));
        Console.WriteLine($"Sent: {i}");
    }
    catch (Exception ex)
    {
        Console.WriteLine($"Send failed: {ex.Message}");
    }
    Thread.Sleep(1000);
}

device.Disconenct();
Console.WriteLine("Done.");

// ── Event listener ───────────────────────────────────────────────────────────

sealed class ConsoleEventListener : EventListener
{
    public void OnConnected()    => Console.WriteLine("Connected!");
    public void OnDisconnected() => Console.WriteLine("Disconnected.");

    public void OnDataReceived(string interfaceName, string path, AstarteVal data) =>
        Console.WriteLine($"Data: {interfaceName}{path} = {data}");

    public void OnObjectReceived(string interfaceName, string path, ObjectEntry[] entries) =>
        Console.WriteLine($"Object: {interfaceName}{path} ({entries.Length} fields)");

    public void OnPropertyReceived(string interfaceName, string path, AstarteVal data) =>
        Console.WriteLine($"Property set: {interfaceName}{path} = {data}");

    public void OnPropertyUnset(string interfaceName, string path) =>
        Console.WriteLine($"Property unset: {interfaceName}{path}");
}
