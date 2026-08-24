package org.astarte.device.internal;

import java.lang.foreign.*;
import java.lang.foreign.MemoryLayout.PathElement;

/**
 * All MemoryLayout constants derived from target/header.h.
 *
 * <p>Layout rules:
 * <ul>
 *   <li>All pointer fields use {@code ValueLayout.ADDRESS} (pointer-sized).</li>
 *   <li>C {@code uintptr_t} maps to {@code ValueLayout.JAVA_LONG} on 64-bit platforms.</li>
 *   <li>C enums are {@code int} → {@code ValueLayout.JAVA_INT}.</li>
 *   <li>Unions are modelled as a single padding block sized to the largest variant.</li>
 * </ul>
 *
 * <p>On a 64-bit system:
 * <ul>
 *   <li>pointer = 8 bytes</li>
 *   <li>int32_t  = 4 bytes</li>
 *   <li>int64_t  = 8 bytes</li>
 *   <li>double   = 8 bytes</li>
 *   <li>bool     = 1 byte</li>
 * </ul>
 */
public final class Layouts {

    private Layouts() {}

    // ── Primitives ──────────────────────────────────────────────────────────

    /** 8-byte pointer (ADDRESS) */
    public static final AddressLayout PTR = ValueLayout.ADDRESS;

    /** 4-byte signed int (C int / int32_t / enum) */
    public static final ValueLayout.OfInt INT32 = ValueLayout.JAVA_INT;

    /** 8-byte signed long (int64_t / uintptr_t on 64-bit) */
    public static final ValueLayout.OfLong INT64 = ValueLayout.JAVA_LONG;

    /** 8-byte double */
    public static final ValueLayout.OfDouble F64 = ValueLayout.JAVA_DOUBLE;

    /** 1-byte boolean */
    public static final ValueLayout.OfBoolean BOOL = ValueLayout.JAVA_BOOLEAN;

    // ── CArray_u8  {ptr, size}  – 16 bytes ──────────────────────────────────
    public static final StructLayout CARRAY_U8 = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_u8");

    // ── CArray_f64 {ptr, size}  – 16 bytes ──────────────────────────────────
    public static final StructLayout CARRAY_F64 = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_f64");

    // ── CArray_i32 {ptr, size}  – 16 bytes ──────────────────────────────────
    public static final StructLayout CARRAY_I32 = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_i32");

    // ── CArray_bool {ptr, size}  – 16 bytes ─────────────────────────────────
    public static final StructLayout CARRAY_BOOL = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_bool");

    // ── CArray_i64 {ptr, size}  – 16 bytes ──────────────────────────────────
    public static final StructLayout CARRAY_I64 = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_i64");

    // ── CStringArray {data, size}  – 16 bytes ───────────────────────────────
    //    const char *const *data  +  uintptr_t size
    public static final StructLayout CSTRING_ARRAY = MemoryLayout.structLayout(
        PTR.withName("data"),
        INT64.withName("size")
    ).withName("CStringArray");

    // ── CArray_CArray_u8 {ptr, size}  – 16 bytes ────────────────────────────
    public static final StructLayout CARRAY_CARRAY_U8 = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_CArray_u8");

    // ── CArray_NativeTimestamp {ptr, size}  – 16 bytes ──────────────────────
    //    NativeTimestamp = int64_t
    public static final StructLayout CARRAY_NATIVE_TIMESTAMP = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_NativeTimestamp");

    // ── NativeDeviceData ─────────────────────────────────────────────────────
    //
    //  struct NativeDeviceData {
    //      NativeDeviceData_Tag tag;   // int  (4 bytes)
    //      // 4 bytes padding to align 8-byte union
    //      union {
    //          double       double_;              // 8 bytes
    //          int32_t      integer;              // 4 bytes  (padded to 8)
    //          bool         boolean;              // 1 byte   (padded to 8)
    //          int64_t      long_integer;         // 8 bytes
    //          const char * string;               // 8 bytes
    //          CArray_u8    binary_blob;          // 16 bytes  ← largest scalar
    //          int64_t      date_time;            // 8 bytes
    //          CArray_f64   double_array;         // 16 bytes
    //          CArray_i32   integer_array;        // 16 bytes
    //          CArray_bool  boolean_array;        // 16 bytes
    //          CArray_i64   long_integer_array;   // 16 bytes
    //          CStringArray string_array;         // 16 bytes
    //          CArray_CArray_u8 binary_blob_array;// 16 bytes
    //          CArray_NativeTimestamp date_time_array; // 16 bytes
    //      };                                     // union = 16 bytes
    //  };
    //  Total: 4 (tag) + 4 (pad) + 16 (union) = 24 bytes

    /** Size of the NativeDeviceData union body in bytes (largest variant = 16). */
    public static final long NATIVE_DEVICE_DATA_UNION_SIZE = 16L;

    public static final StructLayout NATIVE_DEVICE_DATA = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(NATIVE_DEVICE_DATA_UNION_SIZE) // union body
    ).withName("NativeDeviceData");

    /** Byte offset of the tag field in NativeDeviceData. */
    public static final long NATIVE_DEVICE_DATA_TAG_OFFSET = 0L;

    /** Byte offset of the union body in NativeDeviceData. */
    public static final long NATIVE_DEVICE_DATA_UNION_OFFSET = 8L;

    /** Total byte size of NativeDeviceData. */
    public static final long NATIVE_DEVICE_DATA_SIZE = NATIVE_DEVICE_DATA.byteSize(); // 24

    // ── NativeDeviceData tag constants (mirrors NativeDeviceData_Tag enum) ──
    public static final int TAG_DOUBLE           = 0;
    public static final int TAG_INTEGER          = 1;
    public static final int TAG_BOOLEAN          = 2;
    public static final int TAG_LONG_INTEGER     = 3;
    public static final int TAG_STRING           = 4;
    public static final int TAG_BINARY_BLOB      = 5;
    public static final int TAG_DATE_TIME        = 6;
    public static final int TAG_DOUBLE_ARRAY     = 7;
    public static final int TAG_INTEGER_ARRAY    = 8;
    public static final int TAG_BOOLEAN_ARRAY    = 9;
    public static final int TAG_LONG_INTEGER_ARRAY = 10;
    public static final int TAG_STRING_ARRAY     = 11;
    public static final int TAG_BINARY_BLOB_ARRAY = 12;
    public static final int TAG_DATE_TIME_ARRAY  = 13;

    // ── NativeObjectEntry {path, value}  ────────────────────────────────────
    //   const char *path   (8 bytes)
    //   NativeDeviceData value  (24 bytes)
    //   Total: 32 bytes

    public static final StructLayout NATIVE_OBJECT_ENTRY = MemoryLayout.structLayout(
        PTR.withName("path"),
        NATIVE_DEVICE_DATA.withName("value")
    ).withName("NativeObjectEntry");

    public static final long NATIVE_OBJECT_ENTRY_PATH_OFFSET  = 0L;
    public static final long NATIVE_OBJECT_ENTRY_VALUE_OFFSET = 8L;

    // ── CArray_NativeObjectEntry {ptr, size}  – 16 bytes ────────────────────
    public static final StructLayout CARRAY_NATIVE_OBJECT_ENTRY = MemoryLayout.structLayout(
        PTR.withName("data_ptr"),
        INT64.withName("size")
    ).withName("CArray_NativeObjectEntry");

    // ── Individual_Body {NativeDeviceData data, int64_t timestamp}  – 32 bytes
    //   NativeDeviceData (24) + int64_t (8) = 32
    public static final StructLayout INDIVIDUAL_BODY = MemoryLayout.structLayout(
        NATIVE_DEVICE_DATA.withName("data"),
        INT64.withName("timestamp")
    ).withName("Individual_Body");

    // ── Object_Body {CArray_NativeObjectEntry data, int64_t timestamp}  – 24 bytes
    //   CArray_NativeObjectEntry (16) + int64_t (8) = 24
    public static final StructLayout OBJECT_BODY = MemoryLayout.structLayout(
        CARRAY_NATIVE_OBJECT_ENTRY.withName("data"),
        INT64.withName("timestamp")
    ).withName("Object_Body");

    // ── NativeValue ──────────────────────────────────────────────────────────
    //
    //  struct NativeValue {
    //      NativeValue_Tag tag;       // int (4 bytes)
    //      // 4 bytes padding
    //      union {
    //          Individual_Body individual;  // 32 bytes  ← largest
    //          Object_Body     object;      // 24 bytes
    //          struct { NativeDeviceData property_set; }; // 24 bytes
    //      };
    //  };
    //  Total: 4 + 4 (pad) + 32 (union) = 40 bytes

    public static final long NATIVE_VALUE_UNION_SIZE = 32L; // Individual_Body is largest

    public static final StructLayout NATIVE_VALUE = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(NATIVE_VALUE_UNION_SIZE) // union body
    ).withName("NativeValue");

    public static final long NATIVE_VALUE_TAG_OFFSET   = 0L;
    public static final long NATIVE_VALUE_UNION_OFFSET = 8L;

    // NativeValue_Tag constants
    public static final int NATIVE_VALUE_INDIVIDUAL    = 0;
    public static final int NATIVE_VALUE_OBJECT        = 1;
    public static final int NATIVE_VALUE_PROPERTY_SET  = 2;
    public static final int NATIVE_VALUE_PROPERTY_UNSET = 3;

    // ── NativeDeviceEvent {interface, path, data}  ───────────────────────────
    //   const char *interface  (8 bytes)
    //   const char *path       (8 bytes)
    //   NativeValue data       (40 bytes)
    //   Total: 56 bytes

    public static final StructLayout NATIVE_DEVICE_EVENT = MemoryLayout.structLayout(
        PTR.withName("interface"),
        PTR.withName("path"),
        NATIVE_VALUE.withName("data")
    ).withName("NativeDeviceEvent");

    public static final long NATIVE_DEVICE_EVENT_INTERFACE_OFFSET = 0L;
    public static final long NATIVE_DEVICE_EVENT_PATH_OFFSET      = 8L;
    public static final long NATIVE_DEVICE_EVENT_DATA_OFFSET      = 16L;

    // ── NativeStringResult_bool ──────────────────────────────────────────────
    //   int tag (4 bytes) + 4 pad + union { bool ok (1, pad to 8) | ptr err (8) } = 16 bytes

    public static final StructLayout NATIVE_STRING_RESULT_BOOL = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(8) // union: largest is ADDRESS (8 bytes)
    ).withName("NativeStringResult_bool");

    public static final long NSR_BOOL_TAG_OFFSET   = 0L;
    /** Byte offset of the ok (bool) field inside the union of NativeStringResult_bool. */
    public static final long NSR_BOOL_OK_OFFSET    = 8L;
    /** Byte offset of the err (ptr) field inside the union of NativeStringResult_bool. */
    public static final long NSR_BOOL_ERR_OFFSET   = 8L;

    // NativeStringResult_bool tag constants (Ok_bool = 0, Err_bool = 1)
    public static final int NSR_OK  = 0;
    public static final int NSR_ERR = 1;

    // ── NativeStringResult_NativeManuallyDrop_NativeDeviceEvent ─────────────
    //   int tag (4) + 4 pad + union { NativeDeviceEvent ok (56) | ptr err (8) }
    //   Total: 4 + 4 + 56 = 64 bytes

    public static final long NATIVE_DEVICE_EVENT_SIZE = NATIVE_DEVICE_EVENT.byteSize(); // 56

    public static final StructLayout NSR_DEVICE_EVENT = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(NATIVE_DEVICE_EVENT_SIZE) // union
    ).withName("NativeStringResult_NativeDeviceEvent");

    public static final long NSR_EVENT_TAG_OFFSET = 0L;
    public static final long NSR_EVENT_OK_OFFSET  = 8L;
    public static final long NSR_EVENT_ERR_OFFSET = 8L;

    // ── NativeOption_NativeTimestamp ─────────────────────────────────────────
    //   int tag (4) + 4 pad + union { int64_t some (8) } = 16 bytes

    public static final StructLayout NATIVE_OPTION_TIMESTAMP = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        INT64.withName("some")   // sole union variant; always allocated
    ).withName("NativeOption_NativeTimestamp");

    public static final long OPT_TS_TAG_OFFSET  = 0L;
    public static final long OPT_TS_SOME_OFFSET = 8L;
    public static final int  OPT_TS_SOME = 0; // Some_NativeTimestamp
    public static final int  OPT_TS_NONE = 1; // None_NativeTimestamp

    // ── NativeIndividualSend ─────────────────────────────────────────────────
    //   interface (8) + path (8) + NativeDeviceData (24) + NativeOption_NativeTimestamp (16)
    //   = 56 bytes

    public static final StructLayout NATIVE_INDIVIDUAL_SEND = MemoryLayout.structLayout(
        PTR.withName("interface"),
        PTR.withName("path"),
        NATIVE_DEVICE_DATA.withName("data"),
        NATIVE_OPTION_TIMESTAMP.withName("timestamp")
    ).withName("NativeIndividualSend");

    public static final long IND_SEND_INTERFACE_OFFSET  = 0L;
    public static final long IND_SEND_PATH_OFFSET       = 8L;
    public static final long IND_SEND_DATA_OFFSET       = 16L;
    public static final long IND_SEND_TIMESTAMP_OFFSET  = 40L;

    // ── NativeObjectSend ─────────────────────────────────────────────────────
    //   interface (8) + path (8) + CArray_NativeObjectEntry (16) + NativeOption_NativeTimestamp (16)
    //   = 48 bytes

    public static final StructLayout NATIVE_OBJECT_SEND = MemoryLayout.structLayout(
        PTR.withName("interface"),
        PTR.withName("path"),
        CARRAY_NATIVE_OBJECT_ENTRY.withName("data"),
        NATIVE_OPTION_TIMESTAMP.withName("timestamp")
    ).withName("NativeObjectSend");

    public static final long OBJ_SEND_INTERFACE_OFFSET  = 0L;
    public static final long OBJ_SEND_PATH_OFFSET       = 8L;
    public static final long OBJ_SEND_DATA_OFFSET       = 16L;
    public static final long OBJ_SEND_TIMESTAMP_OFFSET  = 32L;

    // ── NativeSetProperty ────────────────────────────────────────────────────
    //   interface (8) + path (8) + NativeDeviceData (24) = 40 bytes

    public static final StructLayout NATIVE_SET_PROPERTY = MemoryLayout.structLayout(
        PTR.withName("interface"),
        PTR.withName("path"),
        NATIVE_DEVICE_DATA.withName("data")
    ).withName("NativeSetProperty");

    public static final long SET_PROP_INTERFACE_OFFSET = 0L;
    public static final long SET_PROP_PATH_OFFSET      = 8L;
    public static final long SET_PROP_DATA_OFFSET      = 16L;

    // ── NativePropertyIdentifier ─────────────────────────────────────────────
    //   interface (8) + path (8) = 16 bytes

    public static final StructLayout NATIVE_PROPERTY_IDENTIFIER = MemoryLayout.structLayout(
        PTR.withName("interface"),
        PTR.withName("path")
    ).withName("NativePropertyIdentifier");

    public static final long PROP_ID_INTERFACE_OFFSET = 0L;
    public static final long PROP_ID_PATH_OFFSET      = 8L;

    // ── NativeOption_NativeDeviceData ────────────────────────────────────────
    //   int tag (4) + 4 pad + union { NativeDeviceData some (24) | – } = 32 bytes

    public static final StructLayout NATIVE_OPTION_DEVICE_DATA = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(NATIVE_DEVICE_DATA_SIZE) // union body (24 bytes)
    ).withName("NativeOption_NativeDeviceData");

    public static final long OPT_DD_TAG_OFFSET  = 0L;
    public static final long OPT_DD_SOME_OFFSET = 8L;
    public static final int  OPT_DD_SOME = 0; // Some_NativeDeviceData
    public static final int  OPT_DD_NONE = 1; // None_NativeDeviceData

    // ── NativeStringResult_NativeOption_NativeDeviceData ────────────────────
    //   int tag (4) + 4 pad + union { NativeOption_NativeDeviceData ok (32) | ptr err (8) }
    //   = 4 + 4 + 32 = 40 bytes

    public static final long NATIVE_OPTION_DEVICE_DATA_SIZE = NATIVE_OPTION_DEVICE_DATA.byteSize(); // 32

    public static final StructLayout NSR_OPTION_DEVICE_DATA = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(NATIVE_OPTION_DEVICE_DATA_SIZE) // union
    ).withName("NativeStringResult_NativeOption_NativeDeviceData");

    public static final long NSR_ODD_TAG_OFFSET = 0L;
    public static final long NSR_ODD_OK_OFFSET  = 8L;
    public static final long NSR_ODD_ERR_OFFSET = 8L;

    // ── NativeMqttConnectionConfig ───────────────────────────────────────────
    //   4 × ptr = 32 bytes

    public static final StructLayout NATIVE_MQTT_CONFIG = MemoryLayout.structLayout(
        PTR.withName("device_id"),
        PTR.withName("cred_secr"),
        PTR.withName("realm"),
        PTR.withName("pairing_url")
    ).withName("NativeMqttConnectionConfig");

    public static final long MQTT_CFG_DEVICE_ID_OFFSET   = 0L;
    public static final long MQTT_CFG_CRED_SECR_OFFSET   = 8L;
    public static final long MQTT_CFG_REALM_OFFSET        = 16L;
    public static final long MQTT_CFG_PAIRING_URL_OFFSET  = 24L;

    // ── NativeGrpcConnectionConfig ───────────────────────────────────────────
    //   1 × ptr = 8 bytes

    public static final StructLayout NATIVE_GRPC_CONFIG = MemoryLayout.structLayout(
        PTR.withName("message_hub_addr")
    ).withName("NativeGrpcConnectionConfig");

    // ── NativeConnectionConfig ───────────────────────────────────────────────
    //   int tag (4) + 4 pad + union { NativeMqttConnectionConfig (32) | NativeGrpcConnectionConfig (8) }
    //   Total: 4 + 4 + 32 = 40 bytes

    public static final StructLayout NATIVE_CONNECTION_CONFIG = MemoryLayout.structLayout(
        INT32.withName("tag"),
        MemoryLayout.paddingLayout(4),
        MemoryLayout.paddingLayout(32L) // union sized to NativeMqttConnectionConfig
    ).withName("NativeConnectionConfig");

    public static final long CONN_CFG_TAG_OFFSET   = 0L;
    public static final long CONN_CFG_UNION_OFFSET = 8L;

    // NativeConnectionConfig_Tag constants (Mqtt = 0, Grpc = 1)
    public static final int CONN_TAG_MQTT = 0;
    public static final int CONN_TAG_GRPC = 1;

    // ── NativeGenericDeviceConfig ─────────────────────────────────────────────
    //   interfaces_dir (8) + channel_size (8, uintptr_t) + writable_dir (8) = 24 bytes

    public static final StructLayout NATIVE_GENERIC_CONFIG = MemoryLayout.structLayout(
        PTR.withName("interfaces_dir"),
        INT64.withName("channel_size"),
        PTR.withName("writable_dir")
    ).withName("NativeGenericDeviceConfig");

    public static final long GEN_CFG_INTERFACES_DIR_OFFSET = 0L;
    public static final long GEN_CFG_CHANNEL_SIZE_OFFSET   = 8L;
    public static final long GEN_CFG_WRITABLE_DIR_OFFSET   = 16L;

    // ── NativeDeviceConfig ───────────────────────────────────────────────────
    //   NativeConnectionConfig (40) + NativeGenericDeviceConfig (24) = 64 bytes

    public static final StructLayout NATIVE_DEVICE_CONFIG = MemoryLayout.structLayout(
        NATIVE_CONNECTION_CONFIG.withName("connection"),
        NATIVE_GENERIC_CONFIG.withName("generic")
    ).withName("NativeDeviceConfig");

    public static final long DEV_CFG_CONNECTION_OFFSET = 0L;
    public static final long DEV_CFG_GENERIC_OFFSET    = NATIVE_CONNECTION_CONFIG.byteSize(); // 40
}
