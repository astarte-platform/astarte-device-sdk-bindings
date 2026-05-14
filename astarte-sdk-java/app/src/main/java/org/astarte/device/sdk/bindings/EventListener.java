package org.astarte.device.sdk.bindings;
/**
 * Foreign languages implement this trait to receive events.
 */
public interface EventListener {
    void onConnected();
    void onDisconnected();
    void onDataReceived(String _interface, String path, AstarteVal data);
}