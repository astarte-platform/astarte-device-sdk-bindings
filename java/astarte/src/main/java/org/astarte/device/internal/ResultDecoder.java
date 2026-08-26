package org.astarte.device.internal;

import org.astarte.device.AstarteException;
import org.astarte.device.DeviceData;
import org.astarte.device.DeviceEvent;

import java.lang.foreign.MemorySegment;
import java.util.Optional;

import static org.astarte.device.internal.Layouts.*;

/**
 * Helper to decode {@code NativeStringResult_*} variants returned from the C API.
 * The C API represents Results as tagged unions: {@code tag == 0} (Ok), {@code tag == 1} (Err).
 * If Err, the union contains a {@code StaticString} (pointer to char) error message.
 */
public final class ResultDecoder {

    private ResultDecoder() {}

    /**
     * Reads a {@code NativeStringResult_bool} from the given segment.
     * @throws AstarteException if the result is Err.
     */
    public static void decodeBoolResult(MemorySegment seg) {
        int tag = seg.get(INT32, NSR_BOOL_TAG_OFFSET);

        if (tag == NSR_ERR) {
            MemorySegment errPtr = seg.get(PTR, NSR_BOOL_ERR_OFFSET).reinterpret(1024);
            String errMsg = errPtr.getString(0);
            throw new AstarteException(errMsg);
        }
        // If ok, it's a bool, but usually we just care that it succeeded.
        // boolean ok = seg.get(BOOL, NSR_BOOL_OK_OFFSET);
    }

    /**
     * Reads a {@code NativeStringResult_NativeOption_NativeDeviceData} from the given segment.
     * @return An Optional containing the DeviceData if present.
     * @throws AstarteException if the result is Err.
     */
    public static Optional<DeviceData> decodePropertyResult(MemorySegment seg) {
        int tag = seg.get(INT32, NSR_ODD_TAG_OFFSET);
        if (tag == NSR_ERR) {
            MemorySegment errPtr = seg.get(PTR, NSR_ODD_ERR_OFFSET);
            String errMsg = errPtr.getString(0);
            throw new AstarteException(errMsg);
        }

        // It is Ok. The payload is NativeOption_NativeDeviceData
        MemorySegment optionSeg = seg.asSlice(NSR_ODD_OK_OFFSET, NATIVE_OPTION_DEVICE_DATA_SIZE);
        int optTag = optionSeg.get(INT32, OPT_DD_TAG_OFFSET);
        if (optTag == OPT_DD_NONE) {
            return Optional.empty();
        } else {
            MemorySegment dataSeg = optionSeg.asSlice(OPT_DD_SOME_OFFSET, NATIVE_DEVICE_DATA_SIZE);
            return Optional.of(DeviceData.readFrom(dataSeg));
        }
    }

    /**
     * Reads a {@code NativeStringResult_NativeDeviceEvent} from the given segment.
     * @return The decoded DeviceEvent.
     * @throws AstarteException if the result is Err.
     */
    public static DeviceEvent decodeEventResult(MemorySegment seg) {
        int tag = seg.get(INT32, NSR_EVENT_TAG_OFFSET);
        if (tag == NSR_ERR) {
            MemorySegment errPtr = seg.get(PTR, NSR_EVENT_ERR_OFFSET);
            String errMsg = errPtr.getString(0);
            throw new AstarteException(errMsg);
        }

        MemorySegment eventSeg = seg.asSlice(NSR_EVENT_OK_OFFSET, NATIVE_DEVICE_EVENT_SIZE);
        return DeviceEvent.readFrom(eventSeg);
    }
}
