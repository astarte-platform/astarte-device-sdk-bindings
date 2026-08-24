package org.astarte.device;

/**
 * Thrown when the native Astarte SDK returns an error string inside a
 * {@code NativeStringResult_*} union.
 */
public class AstarteException extends RuntimeException {

    public AstarteException(String message) {
        super(message);
    }

    public AstarteException(String message, Throwable cause) {
        super(message, cause);
    }
}
