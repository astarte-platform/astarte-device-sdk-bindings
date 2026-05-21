package org.astarte.device.sdk.bindings;

/**
 * Callback interface for receiving events from the Astarte platform.
 *
 * <p>Implement this interface and pass an instance to
 * {@link AstarteDevice#startListening(EventListener)} to be notified of incoming data.
 * All methods are invoked on the thread that called {@code startListening}.
 *
 * <p>Example:
 * <pre>{@code
 * device.startListening(new EventListener() {
 *     public void onConnected() { System.out.println("connected"); }
 *     public void onDisconnected() { System.out.println("disconnected"); }
 *     public void onDataReceived(String iface, String path, AstarteVal data) { ... }
 *     public void onObjectReceived(String iface, String path, List<ObjectEntry> entries) { ... }
 *     public void onPropertyReceived(String iface, String path, AstarteVal data) { ... }
 *     public void onPropertyUnset(String iface, String path) { ... }
 * });
 * }</pre>
 */
public interface EventListener {

    /**
     * Called when the device successfully establishes a session with the Astarte broker.
     */
    void onConnected();

    /**
     * Called when the session with the Astarte broker is lost.
     *
     * <p>After this callback {@link AstarteDevice#startListening} will return.
     */
    void onDisconnected();

    /**
     * Called when a value arrives on an Individual datastream interface.
     *
     * @param _interface fully qualified Astarte interface name
     * @param path       endpoint path within that interface
     * @param data       the received value wrapped in the matching {@link AstarteVal} variant
     */
    void onDataReceived(String _interface, String path, AstarteVal data);

    /**
     * Called when a value arrives on an Object (aggregate) datastream interface.
     *
     * <p>Each entry in {@code entries} corresponds to one field of the object.
     * Fields whose Rust type cannot be mapped to {@link AstarteVal} (e.g. {@code BinaryBlobArray})
     * are silently omitted from the list.
     *
     * @param _interface fully qualified Astarte interface name
     * @param path       base path of the object endpoint
     * @param entries    key-value pairs making up the received object
     */
    void onObjectReceived(String _interface, String path, java.util.List<ObjectEntry> entries);

    /**
     * Called when a server-owned property is set on the device.
     *
     * @param _interface fully qualified Astarte interface name
     * @param path       endpoint path of the property
     * @param data       the new property value wrapped in the matching {@link AstarteVal} variant
     */
    void onPropertyReceived(String _interface, String path, AstarteVal data);

    /**
     * Called when a server-owned property is unset (its value is removed).
     *
     * @param _interface fully qualified Astarte interface name
     * @param path       endpoint path of the property that was unset
     */
    void onPropertyUnset(String _interface, String path);
}