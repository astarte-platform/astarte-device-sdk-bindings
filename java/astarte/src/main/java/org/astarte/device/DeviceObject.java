package org.astarte.device;

import java.util.HashMap;
import java.util.Map;

/**
 * A builder for an Astarte object (a collection of named DeviceData entries).
 */
public final class DeviceObject {

    private final Map<String, DeviceData> entries = new HashMap<>();

    public DeviceObject() {}

    public DeviceObject put(String key, DeviceData value) {
        entries.put(key, value);
        return this;
    }

    public Map<String, DeviceData> getEntries() {
        return entries;
    }
}
