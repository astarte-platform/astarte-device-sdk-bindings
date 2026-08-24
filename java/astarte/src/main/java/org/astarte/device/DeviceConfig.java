package org.astarte.device;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

import static org.astarte.device.internal.Layouts.*;

/**
 * Configuration for connecting an Astarte Device.
 */
public sealed interface DeviceConfig permits DeviceConfig.Mqtt, DeviceConfig.Grpc {

    record Mqtt(String deviceId, String credSecret, String realm, String pairingUrl,
                String interfacesDir, String writableDir, int channelSize)
            implements DeviceConfig {

        public Mqtt(String deviceId, String credSecret, String realm, String pairingUrl,
                String interfacesDir, String writableDir) {
          this(deviceId, credSecret, realm, pairingUrl, interfacesDir, writableDir, 0);
        }

        @Override
        public MemorySegment toNative(Arena arena) {
            MemorySegment seg = arena.allocate(NATIVE_DEVICE_CONFIG);
            
            // Set Connection config (tag = MQTT)
            seg.set(INT32, DEV_CFG_CONNECTION_OFFSET + CONN_CFG_TAG_OFFSET, CONN_TAG_MQTT);
            MemorySegment mqttCfg = seg.asSlice(DEV_CFG_CONNECTION_OFFSET + CONN_CFG_UNION_OFFSET, NATIVE_MQTT_CONFIG.byteSize());
            mqttCfg.set(PTR, MQTT_CFG_DEVICE_ID_OFFSET, arena.allocateFrom(deviceId));
            mqttCfg.set(PTR, MQTT_CFG_CRED_SECR_OFFSET, arena.allocateFrom(credSecret));
            mqttCfg.set(PTR, MQTT_CFG_REALM_OFFSET, arena.allocateFrom(realm));
            mqttCfg.set(PTR, MQTT_CFG_PAIRING_URL_OFFSET, arena.allocateFrom(pairingUrl));

            // Set Generic config
            MemorySegment genCfg = seg.asSlice(DEV_CFG_GENERIC_OFFSET, NATIVE_GENERIC_CONFIG.byteSize());
            genCfg.set(PTR, GEN_CFG_INTERFACES_DIR_OFFSET, arena.allocateFrom(interfacesDir));
            genCfg.set(INT64, GEN_CFG_CHANNEL_SIZE_OFFSET, channelSize);
            genCfg.set(PTR, GEN_CFG_WRITABLE_DIR_OFFSET, writableDir == null ? MemorySegment.NULL : arena.allocateFrom(writableDir));

            return seg;
        }
    }

    record Grpc(String messageHubAddr, String interfacesDir,
                String writableDir, int channelSize)
            implements DeviceConfig {

        @Override
        public MemorySegment toNative(Arena arena) {
            MemorySegment seg = arena.allocate(NATIVE_DEVICE_CONFIG);

            // Set Connection config (tag = GRPC)
            seg.set(INT32, DEV_CFG_CONNECTION_OFFSET + CONN_CFG_TAG_OFFSET, CONN_TAG_GRPC);
            MemorySegment grpcCfg = seg.asSlice(DEV_CFG_CONNECTION_OFFSET + CONN_CFG_UNION_OFFSET, NATIVE_GRPC_CONFIG.byteSize());
            grpcCfg.set(PTR, 0, arena.allocateFrom(messageHubAddr));

            // Set Generic config
            MemorySegment genCfg = seg.asSlice(DEV_CFG_GENERIC_OFFSET, NATIVE_GENERIC_CONFIG.byteSize());
            genCfg.set(PTR, GEN_CFG_INTERFACES_DIR_OFFSET, arena.allocateFrom(interfacesDir));
            genCfg.set(INT64, GEN_CFG_CHANNEL_SIZE_OFFSET, channelSize);
            genCfg.set(PTR, GEN_CFG_WRITABLE_DIR_OFFSET, writableDir == null ? MemorySegment.NULL : arena.allocateFrom(writableDir));

            return seg;
        }
    }

    /**
     * Allocate a NativeDeviceConfig struct in the given Arena and populate it.
     */
    MemorySegment toNative(Arena arena);
}
