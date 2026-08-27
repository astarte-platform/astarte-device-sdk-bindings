package org.astarte.device.internal;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;

/**
 * Thread-safe registry that prevents upcall stubs and UserData sentinels from being
 * garbage-collected while native code may still use them.
 *
 * <h2>UserData pattern</h2>
 * <p>The C API accepts a {@code void *user_data} that it passes back to each callback
 * verbatim.  Because FFM has no built-in "handle" mechanism we use:
 * <ol>
 *   <li>Allocate a tiny 1-byte {@code MemorySegment} from {@link Arena#global()}.</li>
 *   <li>Store {@code address → payload} in {@link #HANDLE_MAP}.</li>
 *   <li>Pass the segment's {@code rawAddress()} as {@code user_data}.</li>
 *   <li>In the upcall, call {@link #popPayload(long)} to retrieve and unregister.</li>
 * </ol>
 *
 * <h2>Upcall stubs</h2>
 * <p>Upcall stubs created by {@code Linker.upcallStub} are {@code MemorySegment}s backed
 * by a native trampoline.  They must stay reachable until the callback fires.  Register
 * them with {@link #registerStub(MemorySegment)} and remove with
 * {@link #unregisterStub(MemorySegment)}.
 */
public final class CallbackRegistry {
    private CallbackRegistry() {}

    /** Map from sentinel address → arbitrary Java payload. */
    private static final ConcurrentHashMap<CallbackHandle, CallbackData<Object>> HANDLE_MAP = new ConcurrentHashMap<>();

    // ── UserData sentinels ────────────────────────────────────────────────────

    /**
     * Allocate a 1-byte sentinel segment from {@link Arena#global()}, store {@code payload}
     * under its address, and return the segment.  Pass the segment directly as
     * {@code user_data} (the FFM linker will pass its raw address to native code).
     */
    public static <T> CallbackHandle registerPayload(CallbackData<T> payload) {
        CallbackHandle handle = CallbackHandle.nextHandle();
        HANDLE_MAP.put(handle, (CallbackData<Object>) payload);
        return handle;
    }

    /**
     * Retrieve and remove the payload stored under {@code address}.  Returns {@code null}
     * if no payload was registered for that address (e.g. double-pop).
     */
    @SuppressWarnings("unchecked")
    public static <T> CallbackData<T> popPayload(MemorySegment fakeHandle) {
        CallbackHandle handle = CallbackHandle.ofMemorySegment(fakeHandle);
        return (CallbackData<T>) HANDLE_MAP.remove(handle);
    }

    public record CallbackData<T>(CompletableFuture<T> future, Arena arena) {}

    public static class CallbackHandle {
        private static final AtomicLong CURRENT_HANDLE = new AtomicLong(0);
        private static final long MAX;
        
        static {
            long ptrBytes = Layouts.PTR.byteSize();
            
            if (ptrBytes == 8) {
                MAX = Long.MAX_VALUE;
            }
            else if (ptrBytes == 4) {
                MAX = (long) Integer.MAX_VALUE;
            }
            else {
                throw new RuntimeException(String.format("size of pointer not supported %d", ptrBytes));
            }
        }

        private final long handle;

        private CallbackHandle(long inHandle) {
            handle = inHandle;
        }

        private static CallbackHandle nextHandle() {
            CallbackHandle newHandle = new CallbackHandle(CURRENT_HANDLE.getAndIncrement());
            return newHandle;
        }

        public static CallbackHandle ofMemorySegment(MemorySegment fakeMemorySegment) {
            long fakeAddress = fakeMemorySegment.address();
            return new CallbackHandle(fakeAddress);
        }

        public MemorySegment fakeMemorySegment() {
            // NOTE this must not be dereferenced
            long fakeAddress = this.handle % MAX;
            return MemorySegment.ofAddress(fakeAddress);
        }

        public int hashCode() {
            return Long.hashCode(this.handle);
        }

        public boolean equals(Object obj) {
            if (obj != null && obj instanceof CallbackHandle) {
                return ((CallbackHandle) obj).handle == this.handle;
            }
            else {
                return false;
            }
        }

        public String toString() {
            return String.format("CallbackHandle(%d)", this.handle);
        }
    }
}
