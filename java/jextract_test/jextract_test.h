#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct DeviceRuntimeHandle DeviceRuntimeHandle;

typedef struct DeviceRuntimeHandle *NativeDeviceHandle;

typedef struct NativeMqttConnectionConfig {
  const char *device_id;
  const char *cred_secr;
  const char *realm;
  const char *pairing_url;
} NativeMqttConnectionConfig;

typedef struct NativeGrpcConnectionConfig {
  const char *message_hub_addr;
} NativeGrpcConnectionConfig;

typedef enum NativeConnectionConfig_Tag {
  Mqtt,
  Grpc,
} NativeConnectionConfig_Tag;

typedef struct NativeConnectionConfig {
  NativeConnectionConfig_Tag tag;
  union {
    struct NativeMqttConnectionConfig mqtt;
    struct NativeGrpcConnectionConfig grpc;
  } value;
} NativeConnectionConfig;

typedef struct NativeGenericDeviceConfig {
  const char *interfaces_dir;
  uintptr_t channel_size;
  const char *writable_dir;
} NativeGenericDeviceConfig;

typedef struct NativeDeviceConfig {
  struct NativeConnectionConfig connection;
  struct NativeGenericDeviceConfig generic;
} NativeDeviceConfig;

typedef const char *StaticString;

typedef enum NativeStringResult_bool_Tag {
  Ok_bool,
  Err_bool,
} NativeStringResult_bool_Tag;

typedef struct NativeStringResult_bool {
  NativeStringResult_bool_Tag tag;
  union {
    bool ok;
    StaticString err;
  } value;
} NativeStringResult_bool;

typedef void *UserData;

typedef void (*DeviceHandleBuildCallback)(const struct NativeStringResult_bool *result,
                                          UserData user_data);

typedef void (*DeviceHandleLoopCallback)(const struct NativeStringResult_bool *result,
                                         UserData user_data);

typedef struct CArray_u8 {
  const uint8_t *data_ptr;
  uintptr_t size;
} CArray_u8;

typedef int64_t NativeTimestamp;

typedef struct CArray_f64 {
  const double *data_ptr;
  uintptr_t size;
} CArray_f64;

typedef struct CArray_i32 {
  const int32_t *data_ptr;
  uintptr_t size;
} CArray_i32;

typedef struct CArray_bool {
  const bool *data_ptr;
  uintptr_t size;
} CArray_bool;

typedef struct CArray_i64 {
  const int64_t *data_ptr;
  uintptr_t size;
} CArray_i64;

typedef struct CStringArray {
  const char *const *data;
  uintptr_t size;
} CStringArray;

typedef struct CArray_CArray_u8 {
  const struct CArray_u8 *data_ptr;
  uintptr_t size;
} CArray_CArray_u8;

typedef struct CArray_NativeTimestamp {
  const NativeTimestamp *data_ptr;
  uintptr_t size;
} CArray_NativeTimestamp;

typedef enum NativeDeviceData_Tag {
  Double,
  Integer,
  Boolean,
  LongInteger,
  String,
  BinaryBlob,
  DateTime,
  DoubleArray,
  IntegerArray,
  BooleanArray,
  LongIntegerArray,
  StringArray,
  BinaryBlobArray,
  DateTimeArray,
} NativeDeviceData_Tag;

typedef struct NativeDeviceData {
  NativeDeviceData_Tag tag;
  union {
    double double_;
    int32_t integer;
    bool boolean;
    int64_t long_integer;
    const char *string;
    struct CArray_u8 binary_blob;
    NativeTimestamp date_time;
    struct CArray_f64 double_array;
    struct CArray_i32 integer_array;
    struct CArray_bool boolean_array;
    struct CArray_i64 long_integer_array;
    struct CStringArray string_array;
    struct CArray_CArray_u8 binary_blob_array;
    struct CArray_NativeTimestamp date_time_array;
  } value;
} NativeDeviceData;

typedef struct NativeObjectEntry {
  const char *path;
  struct NativeDeviceData value;
} NativeObjectEntry;

typedef struct CArray_NativeObjectEntry {
  const struct NativeObjectEntry *data_ptr;
  uintptr_t size;
} CArray_NativeObjectEntry;

typedef enum NativeValue_Tag {
  Individual,
  Object,
  PropertySet,
  PropertyUnset,
} NativeValue_Tag;

typedef struct Individual_Body {
  struct NativeDeviceData data;
  int64_t timestamp;
} Individual_Body;

typedef struct Object_Body {
  struct CArray_NativeObjectEntry data;
  int64_t timestamp;
} Object_Body;

typedef struct NativeValue {
  NativeValue_Tag tag;
  union {
    Individual_Body individual;
    Object_Body object;
    struct NativeDeviceData property_set;
  } value;
} NativeValue;

typedef struct NativeDeviceEvent {
  const char *interface;
  const char *path;
  struct NativeValue data;
} NativeDeviceEvent;

typedef struct NativeDeviceEvent NativeManuallyDrop_NativeDeviceEvent;

typedef enum NativeStringResult_NativeManuallyDrop_NativeDeviceEvent_Tag {
  Ok_NativeManuallyDrop_NativeDeviceEvent,
  Err_NativeManuallyDrop_NativeDeviceEvent,
} NativeStringResult_NativeManuallyDrop_NativeDeviceEvent_Tag;

typedef struct NativeStringResult_NativeManuallyDrop_NativeDeviceEvent {
  NativeStringResult_NativeManuallyDrop_NativeDeviceEvent_Tag tag;
  union {
    NativeManuallyDrop_NativeDeviceEvent ok;
    StaticString err;
  } value;
} NativeStringResult_NativeManuallyDrop_NativeDeviceEvent;

typedef void (*DeviceHandleReceiveCallback)(const struct NativeStringResult_NativeManuallyDrop_NativeDeviceEvent *result,
                                            UserData user_data);

typedef enum NativeOption_NativeTimestamp_Tag {
  Some_NativeTimestamp,
  None_NativeTimestamp,
} NativeOption_NativeTimestamp_Tag;

typedef struct NativeOption_NativeTimestamp {
  NativeOption_NativeTimestamp_Tag tag;
  union {
    NativeTimestamp some;
  } value;
} NativeOption_NativeTimestamp;

typedef struct NativeIndividualSend {
  const char *interface;
  const char *path;
  struct NativeDeviceData data;
  struct NativeOption_NativeTimestamp timestamp;
} NativeIndividualSend;

typedef void (*DeviceHandleSendCallback)(const struct NativeStringResult_bool *result,
                                         UserData user_data);

typedef struct NativeObjectSend {
  const char *interface;
  const char *path;
  struct CArray_NativeObjectEntry data;
  struct NativeOption_NativeTimestamp timestamp;
} NativeObjectSend;

typedef struct NativeSetProperty {
  const char *interface;
  const char *path;
  struct NativeDeviceData data;
} NativeSetProperty;

typedef struct NativePropertyIdentifier {
  const char *interface;
  const char *path;
} NativePropertyIdentifier;

typedef enum NativeOption_NativeDeviceData_Tag {
  Some_NativeDeviceData,
  None_NativeDeviceData,
} NativeOption_NativeDeviceData_Tag;

typedef struct NativeOption_NativeDeviceData {
  NativeOption_NativeDeviceData_Tag tag;
  union {
    struct NativeDeviceData some;
  } value;
} NativeOption_NativeDeviceData;

typedef enum NativeStringResult_NativeOption_NativeDeviceData_Tag {
  Ok_NativeOption_NativeDeviceData,
  Err_NativeOption_NativeDeviceData,
} NativeStringResult_NativeOption_NativeDeviceData_Tag;

typedef struct NativeStringResult_NativeOption_NativeDeviceData {
  NativeStringResult_NativeOption_NativeDeviceData_Tag tag;
  union {
    struct NativeOption_NativeDeviceData ok;
    StaticString err;
  } value;
} NativeStringResult_NativeOption_NativeDeviceData;

typedef void (*DeviceHandleGetCallback)(const struct NativeStringResult_NativeOption_NativeDeviceData *result,
                                        UserData user_data);

typedef void (*DeviceHandleDisconnectCallback)(const struct NativeStringResult_bool *result,
                                               UserData user_data);

NativeDeviceHandle device_handle_init(void);

void device_handle_connect(NativeDeviceHandle handle,
                           const struct NativeDeviceConfig *config,
                           DeviceHandleBuildCallback build_cbk,
                           UserData build_user_data,
                           DeviceHandleLoopCallback loop_cbk,
                           UserData loop_user_data);

void device_handle_receive(NativeDeviceHandle device_handle,
                           DeviceHandleReceiveCallback callback,
                           UserData user_data);

void device_handle_free_device_event(struct NativeDeviceEvent *event);

void device_handle_send_individual(NativeDeviceHandle device_handle,
                                   const struct NativeIndividualSend *data,
                                   DeviceHandleSendCallback callback,
                                   UserData user_data);

void device_handle_send_object(NativeDeviceHandle device_handle,
                               const struct NativeObjectSend *data,
                               DeviceHandleSendCallback callback,
                               UserData user_data);

void device_handle_set_property(NativeDeviceHandle device_handle,
                                const struct NativeSetProperty *property,
                                DeviceHandleSendCallback callback,
                                UserData user_data);

void device_handle_unset_property(NativeDeviceHandle device_handle,
                                  const struct NativePropertyIdentifier *property,
                                  DeviceHandleSendCallback callback,
                                  UserData user_data);

void device_handle_get_property(NativeDeviceHandle device_handle,
                                const struct NativePropertyIdentifier *property,
                                DeviceHandleGetCallback callback,
                                UserData user_data);

void device_handle_free_get_property(struct NativeOption_NativeDeviceData *property);

void device_handle_disconnect(NativeDeviceHandle handle,
                              DeviceHandleDisconnectCallback disconnect_cbk,
                              UserData user_data);

void device_handle_free(NativeDeviceHandle handle);
