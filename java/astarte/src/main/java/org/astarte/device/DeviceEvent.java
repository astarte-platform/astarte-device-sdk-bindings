package org.astarte.device;

import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.util.HashMap;
import java.util.Map;

import static org.astarte.device.internal.Layouts.*;

/**
 * An event received from Astarte.
 */
public record DeviceEvent(String iface, String path, EventValue value) {

    public sealed interface EventValue permits
            EventValue.Individual, EventValue.Object,
            EventValue.PropertySet, EventValue.PropertyUnset {

        record Individual(DeviceData data, long timestampMillis) implements EventValue {}
        record Object(Map<String, DeviceData> entries, long timestampMillis) implements EventValue {}
        record PropertySet(DeviceData data) implements EventValue {}
        record PropertyUnset() implements EventValue {}
    }

    /**
     * Reads a NativeDeviceEvent from the given memory segment.
     */
    public static DeviceEvent readFrom(MemorySegment seg) {
        String iface = seg.get(PTR, NATIVE_DEVICE_EVENT_INTERFACE_OFFSET).getString(0);
        String path = seg.get(PTR, NATIVE_DEVICE_EVENT_PATH_OFFSET).getString(0);
        
        MemorySegment valueSeg = seg.asSlice(NATIVE_DEVICE_EVENT_DATA_OFFSET, NATIVE_VALUE.byteSize());
        int tag = valueSeg.get(INT32, NATIVE_VALUE_TAG_OFFSET);
        MemorySegment unionSeg = valueSeg.asSlice(NATIVE_VALUE_UNION_OFFSET, NATIVE_VALUE_UNION_SIZE);

        EventValue eventValue = switch (tag) {
            case NATIVE_VALUE_INDIVIDUAL -> {
                DeviceData data = DeviceData.readFrom(unionSeg.asSlice(0, NATIVE_DEVICE_DATA_SIZE));
                long timestamp = unionSeg.get(INT64, NATIVE_DEVICE_DATA_SIZE); // timestamp is after NativeDeviceData
                yield new EventValue.Individual(data, timestamp);
            }
            case NATIVE_VALUE_OBJECT -> {
                MemorySegment dataArray = unionSeg.asSlice(0, CARRAY_NATIVE_OBJECT_ENTRY.byteSize());
                MemorySegment entriesPtr = dataArray.get(PTR, 0);
                long size = dataArray.get(INT64, 8);
                Map<String, DeviceData> entriesMap = new HashMap<>();
                for (int i = 0; i < size; i++) {
                    MemorySegment entrySeg = entriesPtr.asSlice(i * NATIVE_OBJECT_ENTRY.byteSize(), NATIVE_OBJECT_ENTRY.byteSize());
                    String entryPath = entrySeg.get(PTR, NATIVE_OBJECT_ENTRY_PATH_OFFSET).getString(0);
                    DeviceData entryData = DeviceData.readFrom(entrySeg.asSlice(NATIVE_OBJECT_ENTRY_VALUE_OFFSET, NATIVE_DEVICE_DATA_SIZE));
                    entriesMap.put(entryPath, entryData);
                }
                long timestamp = unionSeg.get(INT64, CARRAY_NATIVE_OBJECT_ENTRY.byteSize());
                yield new EventValue.Object(entriesMap, timestamp);
            }
            case NATIVE_VALUE_PROPERTY_SET -> {
                DeviceData data = DeviceData.readFrom(unionSeg.asSlice(0, NATIVE_DEVICE_DATA_SIZE));
                yield new EventValue.PropertySet(data);
            }
            case NATIVE_VALUE_PROPERTY_UNSET -> new EventValue.PropertyUnset();
            default -> throw new IllegalStateException("Unknown NativeValue tag: " + tag);
        };

        return new DeviceEvent(iface, path, eventValue);
    }
}
