package org.astarte.device.internal;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
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
    private static final ConcurrentHashMap<Long, Object> HANDLE_MAP = new ConcurrentHashMap<>();

    /** Set of live upcall stubs. */
    private static final CopyOnWriteArrayList<MemorySegment> LIVE_STUBS =
            new CopyOnWriteArrayList<>();

    // ── UserData sentinels ────────────────────────────────────────────────────

    /**
     * Allocate a 1-byte sentinel segment from {@link Arena#global()}, store {@code payload}
     * under its address, and return the segment.  Pass the segment directly as
     * {@code user_data} (the FFM linker will pass its raw address to native code).
     */
    public static MemorySegment registerPayload(Object payload) {
        // FIXME i think this never gets cleared since it's created in the global arena
        MemorySegment sentinel = Arena.global().allocate(1L);
        long addr = sentinel.address();
        HANDLE_MAP.put(addr, payload);
        return sentinel;
    }

    /**
     * Retrieve and remove the payload stored under {@code address}.  Returns {@code null}
     * if no payload was registered for that address (e.g. double-pop).
     */
    @SuppressWarnings("unchecked")
    public static <T> T popPayload(long address) {
        return (T) HANDLE_MAP.remove(address);
    }

    // ── Upcall stubs ─────────────────────────────────────────────────────────

    /** Register an upcall stub to prevent it from being GC'd. */
    public static void registerStub(MemorySegment stub) {
        LIVE_STUBS.add(stub);
    }

    /** Unregister an upcall stub after the callback has been invoked. */
    public static void unregisterStub(MemorySegment stub) {
        LIVE_STUBS.remove(stub);
    }
}
