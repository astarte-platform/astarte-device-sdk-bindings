package org.astarte.device.sdk.bindings

import org.junit.Assert.*
import org.junit.Test

class AstarteConfigTest {

    private val realm = "myrealm"
    private val devId = "device-id-123"
    private val secret = "secret-abc"
    private val pairing = "http://api.astarte.example/pairing"

    private fun cfg(ignoreSsl: Boolean) = AstarteConfig(realm, devId, secret, pairing, ignoreSsl)

    // ── Field access ─────────────────────────────────────────────────────────

    @Test fun fields_are_accessible() {
        val c = cfg(false)
        assertEquals(realm, c.realm)
        assertEquals(devId, c.deviceId)
        assertEquals(secret, c.credentialsSecret)
        assertEquals(pairing, c.pairingUrl)
        assertFalse(c.ignoreSsl)
    }

    @Test fun ignore_ssl_true() {
        assertTrue(cfg(true).ignoreSsl)
    }

    @Test fun ignore_ssl_false() {
        assertFalse(cfg(false).ignoreSsl)
    }

    // ── equals ───────────────────────────────────────────────────────────────

    @Test fun equals_same_instance() {
        val c = cfg(false)
        assertEquals(c, c)
    }

    @Test fun equals_two_equal_instances() {
        assertEquals(cfg(false), cfg(false))
    }

    @Test fun not_equals_different_realm() {
        assertNotEquals(
            AstarteConfig("realm-a", devId, secret, pairing, false),
            AstarteConfig("realm-b", devId, secret, pairing, false)
        )
    }

    @Test fun not_equals_different_device_id() {
        assertNotEquals(
            AstarteConfig(realm, "dev-a", secret, pairing, false),
            AstarteConfig(realm, "dev-b", secret, pairing, false)
        )
    }

    @Test fun not_equals_different_secret() {
        assertNotEquals(
            AstarteConfig(realm, devId, "secret-1", pairing, false),
            AstarteConfig(realm, devId, "secret-2", pairing, false)
        )
    }

    @Test fun not_equals_different_pairing_url() {
        assertNotEquals(
            AstarteConfig(realm, devId, secret, "http://host-a/pairing", false),
            AstarteConfig(realm, devId, secret, "http://host-b/pairing", false)
        )
    }

    @Test fun not_equals_different_ignore_ssl() {
        assertNotEquals(cfg(true), cfg(false))
    }

    @Test fun not_equals_null() {
        assertNotEquals(null, cfg(false))
    }

    // ── hashCode ─────────────────────────────────────────────────────────────

    @Test fun hashcode_consistent_with_equals() {
        assertEquals(cfg(true).hashCode(), cfg(true).hashCode())
    }

    @Test fun equal_configs_have_same_hashcode() {
        assertEquals(cfg(false).hashCode(), cfg(false).hashCode())
    }

    // ── toString ─────────────────────────────────────────────────────────────

    @Test fun toString_contains_realm() {
        assertTrue(cfg(false).toString().contains(realm))
    }

    @Test fun toString_contains_device_id() {
        assertTrue(cfg(false).toString().contains(devId))
    }
}
