package org.astarte.device;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;

import static org.astarte.device.internal.Layouts.*;

/**
 * Representation of a value that can be sent to or received from Astarte.
 */
public sealed interface DeviceData permits
        DeviceData.DoubleValue, DeviceData.IntegerValue, DeviceData.BooleanValue,
        DeviceData.LongIntegerValue, DeviceData.StringValue, DeviceData.BinaryBlob,
        DeviceData.DateTime, DeviceData.DoubleArray, DeviceData.IntegerArray,
        DeviceData.BooleanArray, DeviceData.LongIntegerArray, DeviceData.StringArray,
        DeviceData.BinaryBlobArray, DeviceData.DateTimeArray {

    record DoubleValue(double value) implements DeviceData {}
    record IntegerValue(int value) implements DeviceData {}
    record BooleanValue(boolean value) implements DeviceData {}
    record LongIntegerValue(long value) implements DeviceData {}
    record StringValue(String value) implements DeviceData {}
    record BinaryBlob(byte[] value) implements DeviceData {}
    record DateTime(Instant value) implements DeviceData {}
    record DoubleArray(double[] value) implements DeviceData {}
    record IntegerArray(int[] value) implements DeviceData {}
    record BooleanArray(boolean[] value) implements DeviceData {}
    record LongIntegerArray(long[] value) implements DeviceData {}
    record StringArray(String[] value) implements DeviceData {}
    record BinaryBlobArray(byte[][] value) implements DeviceData {}
    record DateTimeArray(Instant[] value) implements DeviceData {}

    /**
     * Write this value into a pre-allocated NativeDeviceData segment.
     */
    default void writeTo(MemorySegment seg, Arena arena) {
        if (this instanceof DoubleValue(double v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_DOUBLE);
            seg.set(F64, NATIVE_DEVICE_DATA_UNION_OFFSET, v);
        } else if (this instanceof IntegerValue(int v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_INTEGER);
            seg.set(INT32, NATIVE_DEVICE_DATA_UNION_OFFSET, v);
        } else if (this instanceof BooleanValue(boolean v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_BOOLEAN);
            seg.set(BOOL, NATIVE_DEVICE_DATA_UNION_OFFSET, v);
        } else if (this instanceof LongIntegerValue(long v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_LONG_INTEGER);
            seg.set(INT64, NATIVE_DEVICE_DATA_UNION_OFFSET, v);
        } else if (this instanceof StringValue(String v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_STRING);
            seg.set(PTR, NATIVE_DEVICE_DATA_UNION_OFFSET, arena.allocateFrom(v));
        } else if (this instanceof BinaryBlob(byte[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_BINARY_BLOB);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_U8.byteSize());
            arraySeg.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_BYTE, v));
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof DateTime(Instant v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_DATE_TIME);
            seg.set(INT64, NATIVE_DEVICE_DATA_UNION_OFFSET, v.toEpochMilli());
        } else if (this instanceof DoubleArray(double[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_DOUBLE_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_F64.byteSize());
            arraySeg.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_DOUBLE, v));
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof IntegerArray(int[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_INTEGER_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_I32.byteSize());
            arraySeg.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_INT, v));
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof BooleanArray(boolean[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_BOOLEAN_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_BOOL.byteSize());
            // Need to write booleans to an array. FFM boolean arrays can be tricky if they differ from C bool size (1 byte).
            // ValueLayout.JAVA_BOOLEAN is 1 byte, same as C bool on most platforms.
            MemorySegment boolArray = arena.allocate(v.length);
            for (int i = 0; i < v.length; i++) {
                boolArray.set(ValueLayout.JAVA_BOOLEAN, i, v[i]);
            }
            arraySeg.set(PTR, 0, boolArray);
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof LongIntegerArray(long[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_LONG_INTEGER_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_I64.byteSize());
            arraySeg.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_LONG, v));
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof StringArray(String[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_STRING_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CSTRING_ARRAY.byteSize());
            MemorySegment ptrArray = arena.allocate(ValueLayout.ADDRESS, v.length);
            for (int i = 0; i < v.length; i++) {
                ptrArray.setAtIndex(ValueLayout.ADDRESS, i, arena.allocateFrom(v[i]));
            }
            arraySeg.set(PTR, 0, ptrArray);
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof BinaryBlobArray(byte[][] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_BINARY_BLOB_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_CARRAY_U8.byteSize());
            MemorySegment structArray = arena.allocate(CARRAY_U8, v.length);
            for (int i = 0; i < v.length; i++) {
                MemorySegment elemStruct = structArray.asSlice(i * CARRAY_U8.byteSize(), CARRAY_U8.byteSize());
                elemStruct.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_BYTE, v[i]));
                elemStruct.set(INT64, 8, v[i].length);
            }
            arraySeg.set(PTR, 0, structArray);
            arraySeg.set(INT64, 8, v.length);
        } else if (this instanceof DateTimeArray(Instant[] v)) {
            seg.set(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET, TAG_DATE_TIME_ARRAY);
            MemorySegment arraySeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, CARRAY_NATIVE_TIMESTAMP.byteSize());
            long[] timestamps = new long[v.length];
            for (int i = 0; i < v.length; i++) {
                timestamps[i] = v[i].toEpochMilli();
            }
            arraySeg.set(PTR, 0, arena.allocateFrom(ValueLayout.JAVA_LONG, timestamps));
            arraySeg.set(INT64, 8, v.length);
        } else {
            throw new IllegalArgumentException("Unsupported DeviceData type: " + this.getClass());
        }
    }

    /**
     * Read a DeviceData from a NativeDeviceData segment.
     */
    static DeviceData readFrom(MemorySegment seg) {
        int tag = seg.get(INT32, NATIVE_DEVICE_DATA_TAG_OFFSET);
        MemorySegment unionSeg = seg.asSlice(NATIVE_DEVICE_DATA_UNION_OFFSET, NATIVE_DEVICE_DATA_UNION_SIZE);

        return switch (tag) {
            case TAG_DOUBLE -> new DoubleValue(unionSeg.get(F64, 0));
            case TAG_INTEGER -> new IntegerValue(unionSeg.get(INT32, 0));
            case TAG_BOOLEAN -> new BooleanValue(unionSeg.get(BOOL, 0));
            case TAG_LONG_INTEGER -> new LongIntegerValue(unionSeg.get(INT64, 0));
            case TAG_STRING -> new StringValue(unionSeg.get(PTR, 0).getString(0));
            case TAG_BINARY_BLOB -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                yield new BinaryBlob(ptr.asSlice(0, size).toArray(ValueLayout.JAVA_BYTE));
            }
            case TAG_DATE_TIME -> new DateTime(Instant.ofEpochMilli(unionSeg.get(INT64, 0)));
            case TAG_DOUBLE_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                yield new DoubleArray(ptr.asSlice(0, size * 8).toArray(ValueLayout.JAVA_DOUBLE));
            }
            case TAG_INTEGER_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                yield new IntegerArray(ptr.asSlice(0, size * 4).toArray(ValueLayout.JAVA_INT));
            }
            case TAG_BOOLEAN_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                byte[] bytes = ptr.asSlice(0, size).toArray(ValueLayout.JAVA_BYTE);
                boolean[] bools = new boolean[(int) size];
                for (int i = 0; i < size; i++) {
                    bools[i] = bytes[i] != 0;
                }
                yield new BooleanArray(bools);
            }
            case TAG_LONG_INTEGER_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                yield new LongIntegerArray(ptr.asSlice(0, size * 8).toArray(ValueLayout.JAVA_LONG));
            }
            case TAG_STRING_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                String[] strings = new String[(int) size];
                for (int i = 0; i < size; i++) {
                    MemorySegment strPtr = ptr.getAtIndex(ValueLayout.ADDRESS, i);
                    strings[i] = strPtr.getString(0);
                }
                yield new StringArray(strings);
            }
            case TAG_BINARY_BLOB_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                byte[][] blobs = new byte[(int) size][];
                for (int i = 0; i < size; i++) {
                    MemorySegment elemStruct = ptr.asSlice((long) i * CARRAY_U8.byteSize(), CARRAY_U8.byteSize());
                    MemorySegment blobPtr = elemStruct.get(PTR, 0);
                    long blobSize = elemStruct.get(INT64, 8);
                    blobs[i] = blobPtr.asSlice(0, blobSize).toArray(ValueLayout.JAVA_BYTE);
                }
                yield new BinaryBlobArray(blobs);
            }
            case TAG_DATE_TIME_ARRAY -> {
                MemorySegment ptr = unionSeg.get(PTR, 0);
                long size = unionSeg.get(INT64, 8);
                long[] longs = ptr.asSlice(0, size * 8).toArray(ValueLayout.JAVA_LONG);
                Instant[] instants = new Instant[(int) size];
                for (int i = 0; i < size; i++) {
                    instants[i] = Instant.ofEpochMilli(longs[i]);
                }
                yield new DateTimeArray(instants);
            }
            default -> throw new IllegalStateException("Unknown NativeDeviceData tag: " + tag);
        };
    }
}
