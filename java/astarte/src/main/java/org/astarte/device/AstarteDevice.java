package org.astarte.device;

import org.astarte.device.internal.CallbackRegistry;
import org.astarte.device.internal.CallbackRegistry.CallbackData;
import org.astarte.device.internal.CallbackRegistry.CallbackHandle;
import org.astarte.device.internal.ResultDecoder;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.time.Instant;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

import static org.astarte.device.internal.Layouts.*;

/**
 * Main Device client class for Astarte Device SDK bindings.
 */
public class AstarteDevice implements AutoCloseable {
    private final MemorySegment ptr;
    private CompletableFuture<Void> loopFuture;

    public AstarteDevice() {
        try {
            this.ptr = (MemorySegment) NativeLoader.DEVICE_HANDLE_INIT.invokeExact();
        } catch (Throwable t) {
            throw new RuntimeException("Failed to initialize AstarteDevice", t);
        }
    }

    /**
     * Connect the device using the provided configuration.
     */
    public synchronized CompletableFuture<Void> connect(DeviceConfig config) {
        if (this.loopFuture != null) {
            throw new IllegalStateException("already connected previously");
        }

        
        Arena connectArena = Arena.ofShared();
        MemorySegment configSeg = config.toNative(connectArena);
        CallbackData<Void> connectData = new CallbackData<>(new CompletableFuture<>(), connectArena);
        // Build callback
        CallbackHandle buildSentinel = CallbackRegistry.registerPayload(connectData);
        MemorySegment buildStub = NativeLoader.LINKER.upcallStub(DeviceCallbacks.CONNECT_CBK,
                NativeLoader.CALLBACK_BOOL_DESC, connectArena);

        // Loop callback
        Arena loopArena = Arena.ofShared();
        this.loopFuture = new CompletableFuture<>();
        CallbackData<Void> loopData = new CallbackData<>(this.loopFuture, loopArena);
        CallbackHandle loopSentinel = CallbackRegistry.registerPayload(loopData);
        MemorySegment loopStub = NativeLoader.LINKER.upcallStub(DeviceCallbacks.LOOP_CBK,
                NativeLoader.CALLBACK_BOOL_DESC, loopArena);

        try {
            NativeLoader.DEVICE_HANDLE_CONNECT.invokeExact(ptr, configSeg, buildStub, buildSentinel.fakeMemorySegment(), loopStub, loopSentinel.fakeMemorySegment());
        } catch (Throwable t) {
            connectData.future().completeExceptionally(t);
            connectArena.close();
            this.loopFuture.completeExceptionally(t);
            loopArena.close();
        }

        return connectData.future();
    }

    /**
     * Disconnect the device.
     */
    public CompletableFuture<Void> disconnect() {
        Arena disconnectArena = Arena.ofShared();
        CallbackData<Void> disconnectData = new CallbackData<>(new CompletableFuture<>(), disconnectArena);
        CallbackHandle sentinel = CallbackRegistry.registerPayload(disconnectData);
        MemorySegment stub = NativeLoader.LINKER.upcallStub(
                DeviceCallbacks.BOOL_CBK,
                NativeLoader.CALLBACK_BOOL_DESC, disconnectArena);

        try {
            NativeLoader.DEVICE_HANDLE_DISCONNECT.invokeExact(ptr, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            disconnectData.future().completeExceptionally(t);
            disconnectArena.close();
        }

        return disconnectData.future();
    }

    /**
     * Wait for and receive the next event from the Astarte cluster.
     */
    public CompletableFuture<DeviceEvent> receiveData() {
        Arena receiveArena = Arena.ofShared();
        CallbackData<DeviceEvent> receiveData = new CallbackData<>(new CompletableFuture<>(), receiveArena);
        CallbackHandle sentinel = CallbackRegistry.registerPayload(receiveData);
        MemorySegment stub = NativeLoader.LINKER.upcallStub(
                DeviceCallbacks.RECEIVE_CBK,
                NativeLoader.CALLBACK_EVENT_DESC, receiveArena);

        try {
            NativeLoader.DEVICE_HANDLE_RECEIVE.invokeExact(ptr, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            receiveData.future().completeExceptionally(t);
            receiveArena.close();
        }
        return receiveData.future();
    }

    /**
     * Send an individual datastream value to Astarte.
     */
    public CompletableFuture<Void> sendIndividual(String iface, String path, DeviceData data, Instant timestamp) {
        Arena sendArena = Arena.ofShared();
        CallbackData<Void> sendData = new CallbackData<>(new CompletableFuture<>(), sendArena);
        
        try {
            MemorySegment individualData = sendArena.allocate(NATIVE_INDIVIDUAL_SEND);
            individualData.set(PTR, IND_SEND_INTERFACE_OFFSET, sendArena.allocateFrom(iface));
            individualData.set(PTR, IND_SEND_PATH_OFFSET, sendArena.allocateFrom(path));
            
            MemorySegment dataSeg = individualData.asSlice(IND_SEND_DATA_OFFSET, NATIVE_DEVICE_DATA_SIZE);
            data.writeTo(dataSeg, sendArena);
            
            MemorySegment tsSeg = individualData.asSlice(IND_SEND_TIMESTAMP_OFFSET, NATIVE_OPTION_TIMESTAMP.byteSize());
            if (timestamp == null) {
                tsSeg.set(INT32, OPT_TS_TAG_OFFSET, OPT_TS_NONE);
            } else {
                tsSeg.set(INT32, OPT_TS_TAG_OFFSET, OPT_TS_SOME);
                tsSeg.set(INT64, OPT_TS_SOME_OFFSET, timestamp.toEpochMilli());
            }

            CallbackHandle sentinel = CallbackRegistry.registerPayload(sendData);
            MemorySegment stub = NativeLoader.LINKER.upcallStub(
                    DeviceCallbacks.SEND_CBK,
                    NativeLoader.CALLBACK_BOOL_DESC, sendArena);
            
            NativeLoader.DEVICE_HANDLE_SEND_INDIVIDUAL.invokeExact(ptr, individualData, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            sendArena.close();
            sendData.future().completeExceptionally(t);
        }
        
        return sendData.future();
    }

    /**
     * Send an aggregate object value to Astarte.
     */
    public CompletableFuture<Void> sendObject(String iface, String path, DeviceObject obj, Instant timestamp) {
        Arena sendArena = Arena.ofShared();
        CallbackData<Void> sendData = new CallbackData<>(new CompletableFuture<>(), sendArena);
        
        try {
            MemorySegment objectData = sendArena.allocate(NATIVE_OBJECT_SEND);
            objectData.set(PTR, OBJ_SEND_INTERFACE_OFFSET, sendArena.allocateFrom(iface));
            objectData.set(PTR, OBJ_SEND_PATH_OFFSET, sendArena.allocateFrom(path));
            
            MemorySegment arraySeg = objectData.asSlice(OBJ_SEND_DATA_OFFSET, CARRAY_NATIVE_OBJECT_ENTRY.byteSize());
            Map<String, DeviceData> entries = obj.getEntries();
            
            MemorySegment structArray = sendArena.allocate(NATIVE_OBJECT_ENTRY, entries.size());
            int i = 0;
            for (Map.Entry<String, DeviceData> entry : entries.entrySet()) {
                MemorySegment elem = structArray.asSlice(i * NATIVE_OBJECT_ENTRY.byteSize(), NATIVE_OBJECT_ENTRY.byteSize());
                elem.set(PTR, NATIVE_OBJECT_ENTRY_PATH_OFFSET, sendArena.allocateFrom(entry.getKey()));
                MemorySegment valueSeg = elem.asSlice(NATIVE_OBJECT_ENTRY_VALUE_OFFSET, NATIVE_DEVICE_DATA_SIZE);
                entry.getValue().writeTo(valueSeg, sendArena);
                i++;
            }
            arraySeg.set(PTR, 0, structArray);
            arraySeg.set(INT64, 8, entries.size());
            
            MemorySegment tsSeg = objectData.asSlice(OBJ_SEND_TIMESTAMP_OFFSET, NATIVE_OPTION_TIMESTAMP.byteSize());
            if (timestamp == null) {
                tsSeg.set(INT32, OPT_TS_TAG_OFFSET, OPT_TS_NONE);
            } else {
                tsSeg.set(INT32, OPT_TS_TAG_OFFSET, OPT_TS_SOME);
                tsSeg.set(INT64, OPT_TS_SOME_OFFSET, timestamp.toEpochMilli());
            }

            CallbackHandle sentinel = CallbackRegistry.registerPayload(sendData);
            MemorySegment stub = NativeLoader.LINKER.upcallStub(
                    DeviceCallbacks.SEND_CBK,
                    NativeLoader.CALLBACK_BOOL_DESC, sendArena);
            
            NativeLoader.DEVICE_HANDLE_SEND_OBJECT.invokeExact(ptr, objectData, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            sendArena.close();
            sendData.future().completeExceptionally(t);
        }
        
        return sendData.future();
    }

    /**
     * Set a property value on an Astarte property interface.
     */
    public CompletableFuture<Void> setProperty(String iface, String path, DeviceData value) {
        Arena sendArena = Arena.ofShared();
        CallbackData<Void> sendData = new CallbackData<>(new CompletableFuture<>(), sendArena);
        
        try {
            MemorySegment propertyData = sendArena.allocate(NATIVE_SET_PROPERTY);
            propertyData.set(PTR, SET_PROP_INTERFACE_OFFSET, sendArena.allocateFrom(iface));
            propertyData.set(PTR, SET_PROP_PATH_OFFSET, sendArena.allocateFrom(path));
            
            MemorySegment dataSeg = propertyData.asSlice(SET_PROP_DATA_OFFSET, NATIVE_DEVICE_DATA_SIZE);
            value.writeTo(dataSeg, sendArena);
            
            CallbackHandle sentinel = CallbackRegistry.registerPayload(sendData);
            MemorySegment stub = NativeLoader.LINKER.upcallStub(
                    DeviceCallbacks.SEND_CBK,
                    NativeLoader.CALLBACK_BOOL_DESC, sendArena);
            
            NativeLoader.DEVICE_HANDLE_SET_PROPERTY.invokeExact(ptr, propertyData, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            sendArena.close();
            sendData.future().completeExceptionally(t);
        }

        return sendData.future();
    }

    /**
     * Unset a property value on an Astarte property interface.
     */
    public CompletableFuture<Void> unsetProperty(String iface, String path) {
        Arena sendArena = Arena.ofShared();
        CallbackData<Void> sendData = new CallbackData<>(new CompletableFuture<>(), sendArena);
        
        try {
            MemorySegment propertyData = sendArena.allocate(NATIVE_PROPERTY_IDENTIFIER);
            propertyData.set(PTR, PROP_ID_INTERFACE_OFFSET, sendArena.allocateFrom(iface));
            propertyData.set(PTR, PROP_ID_PATH_OFFSET, sendArena.allocateFrom(path));
            
            CallbackHandle sentinel = CallbackRegistry.registerPayload(sendData);
            MemorySegment stub = NativeLoader.LINKER.upcallStub(
                    DeviceCallbacks.SEND_CBK,
                    NativeLoader.CALLBACK_BOOL_DESC, sendArena);
            
            NativeLoader.DEVICE_HANDLE_UNSET_PROPERTY.invokeExact(ptr, propertyData, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            sendArena.close();
            sendData.future().completeExceptionally(t);
        }

        return sendData.future();
    }

    /**
     * Retrieve a stored property value from an Astarte property interface.
     */
    public CompletableFuture<Optional<DeviceData>> getProperty(String iface, String path) {
        Arena getArena = Arena.ofShared();
        CallbackData<Optional<DeviceData>> getData = new CallbackData<>(new CompletableFuture<>(), getArena);
        
        try {
            MemorySegment propertyData = getArena.allocate(NATIVE_PROPERTY_IDENTIFIER);
            propertyData.set(PTR, PROP_ID_INTERFACE_OFFSET, getArena.allocateFrom(iface));
            propertyData.set(PTR, PROP_ID_PATH_OFFSET, getArena.allocateFrom(path));
            CallbackHandle sentinel = CallbackRegistry.registerPayload(getData);
            MemorySegment stub = NativeLoader.LINKER.upcallStub(
                    DeviceCallbacks.GET_PROPERTY_CBK,
                    NativeLoader.CALLBACK_GET_PROP_DESC, getArena);
            
            NativeLoader.DEVICE_HANDLE_GET_PROPERTY.invokeExact(ptr, propertyData, stub, sentinel.fakeMemorySegment());
        } catch (Throwable t) {
            getArena.close();
            getData.future().completeExceptionally(t);
        }
        
        return getData.future();
    }

    @Override
    public void close() {
        try {
            NativeLoader.DEVICE_HANDLE_FREE.invokeExact(ptr);
        } catch (Throwable t) {
            throw new RuntimeException("Failed to free AstarteDevice", t);
        }
    }

    private record PayloadWithArenaGeneric<T>(CompletableFuture<T> future, Arena arena) {}

    class DeviceCallbacks {
        static final MethodHandle CONNECT_CBK;
        static final MethodHandle LOOP_CBK;
        static final MethodHandle BOOL_CBK;
        static final MethodHandle RECEIVE_CBK;
        static final MethodHandle SEND_CBK;
        static final MethodHandle GET_PROPERTY_CBK;

        static {
            try {
                CONNECT_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "connectCbk",
                    NativeLoader.CALLBACK_BOOL_DESC.toMethodType());
                LOOP_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "loopCbk",
                    NativeLoader.CALLBACK_BOOL_DESC.toMethodType());
                BOOL_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "boolCbk",
                    NativeLoader.CALLBACK_BOOL_DESC.toMethodType());
                RECEIVE_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "receiveCbk",
                        NativeLoader.CALLBACK_EVENT_DESC.toMethodType());
                SEND_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "sendCbk",
                        NativeLoader.CALLBACK_BOOL_DESC.toMethodType());
                GET_PROPERTY_CBK = MethodHandles.lookup().findStatic(DeviceCallbacks.class, "getPropertyCbk",
                        NativeLoader.CALLBACK_GET_PROP_DESC.toMethodType());
            }
            catch(NoSuchMethodException e) {
                throw new RuntimeException("can't find static method", e);
            }
            catch(IllegalAccessException e) {
                throw new RuntimeException("can't access static method", e);
            }
        }

        static void connectCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<Void> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }

            try {
                ResultDecoder.decodeBoolResult(resultSeg);
                payload.future().complete(null);
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            }
            finally {
                payload.arena().close();
            }
        }

        static void loopCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<Void> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }
            try {
                ResultDecoder.decodeBoolResult(resultSeg);
                payload.future().complete(null);
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            }
            finally {
                payload.arena().close();
            }
        }

        static void boolCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<Void> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }
            try {
                ResultDecoder.decodeBoolResult(resultSeg);
                payload.future().complete(null);
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            }
            finally {
                payload.arena().close();
            }
        }

        static void receiveCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<DeviceEvent> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }
            try {
                DeviceEvent event = ResultDecoder.decodeEventResult(resultSeg);
                payload.future().complete(event);
            
                // We need to free the event if it was successful.
                // resultSeg is a pointer to NativeStringResult_NativeManuallyDrop_NativeDeviceEvent
                // The OK variant holds a NativeDeviceEvent inline.
                // We can call device_handle_free_device_event on a pointer to it.
                // But doing it via FFM needs care to not leak.
                // To simplify, if we decoded it successfully, we should call the free function.
                // The NativeDeviceEvent is at offset NSR_EVENT_OK_OFFSET.
                MemorySegment eventPtr = resultSeg.asSlice(NSR_EVENT_OK_OFFSET, NATIVE_DEVICE_EVENT_SIZE);
                NativeLoader.DEVICE_HANDLE_FREE_DEVICE_EVENT.invokeExact(eventPtr);
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            }
            finally {
                payload.arena().close();
            }
        }

        static void sendCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<Void> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }
            try {
                ResultDecoder.decodeBoolResult(resultSeg);
                payload.future().complete(null);
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            } finally {
                payload.arena().close();
            }
        }

        static void getPropertyCbk(MemorySegment resultSeg, MemorySegment userData) {
            CallbackData<Optional<DeviceData>> payload = CallbackRegistry.popPayload(userData);

            if (payload == null) {
                throw new AstarteException("handle to userData received was invalid");
            }
            try {
                Optional<DeviceData> data = ResultDecoder.decodePropertyResult(resultSeg);
                payload.future().complete(data);
            
                // Need to free NativeOption_NativeDeviceData
                MemorySegment optPtr = resultSeg.asSlice(NSR_ODD_OK_OFFSET, NATIVE_OPTION_DEVICE_DATA_SIZE);
                NativeLoader.DEVICE_HANDLE_FREE_GET_PROPERTY.invokeExact(optPtr);
            
            } catch (Throwable t) {
                payload.future().completeExceptionally(t);
            } finally {
                payload.arena().close();
            }
        }
    }
}
