using AstarteDeviceSdkBindings;
using Xunit;

namespace AstarteSdk.Tests;

public class AstarteConfigTests
{
    private const string Realm   = "myrealm";
    private const string DevId   = "device-id-123";
    private const string Secret  = "secret-abc";
    private const string Pairing = "http://api.astarte.example/pairing";

    private static AstarteConfig Cfg(bool ignoreSsl) =>
        new AstarteConfig(Realm, DevId, Secret, Pairing, ignoreSsl);

    // ── Field access ─────────────────────────────────────────────────────────

    [Fact] public void Fields_AreAccessible()
    {
        var c = Cfg(false);
        Assert.Equal(Realm,   c.Realm);
        Assert.Equal(DevId,   c.DeviceId);
        Assert.Equal(Secret,  c.CredentialsSecret);
        Assert.Equal(Pairing, c.PairingUrl);
        Assert.False(c.IgnoreSsl);
    }

    [Fact] public void IgnoreSsl_True()  => Assert.True(Cfg(true).IgnoreSsl);
    [Fact] public void IgnoreSsl_False() => Assert.False(Cfg(false).IgnoreSsl);

    // ── Equals ───────────────────────────────────────────────────────────────

    [Fact] public void Equals_SameInstance() { var c = Cfg(false); Assert.Equal(c, c); }

    [Fact] public void Equals_TwoEqualInstances() => Assert.Equal(Cfg(false), Cfg(false));

    [Fact] public void NotEquals_DifferentRealm() =>
        Assert.NotEqual(
            new AstarteConfig("a", DevId, Secret, Pairing, false),
            new AstarteConfig("b", DevId, Secret, Pairing, false));

    [Fact] public void NotEquals_DifferentDeviceId() =>
        Assert.NotEqual(
            new AstarteConfig(Realm, "dev-a", Secret, Pairing, false),
            new AstarteConfig(Realm, "dev-b", Secret, Pairing, false));

    [Fact] public void NotEquals_DifferentSecret() =>
        Assert.NotEqual(
            new AstarteConfig(Realm, DevId, "s1", Pairing, false),
            new AstarteConfig(Realm, DevId, "s2", Pairing, false));

    [Fact] public void NotEquals_DifferentPairingUrl() =>
        Assert.NotEqual(
            new AstarteConfig(Realm, DevId, Secret, "http://a/pairing", false),
            new AstarteConfig(Realm, DevId, Secret, "http://b/pairing", false));

    [Fact] public void NotEquals_DifferentIgnoreSsl() =>
        Assert.NotEqual(Cfg(true), Cfg(false));

    [Fact] public void NotEquals_Null() => Assert.False(Cfg(false).Equals((object?)null));

    // ── GetHashCode ───────────────────────────────────────────────────────────

    [Fact] public void Hashcode_ConsistentWithEquals() =>
        Assert.Equal(Cfg(true).GetHashCode(), Cfg(true).GetHashCode());

    [Fact] public void EqualConfigs_SameHashcode() =>
        Assert.Equal(Cfg(false).GetHashCode(), Cfg(false).GetHashCode());

    // ── ToString ─────────────────────────────────────────────────────────────

    [Fact] public void ToString_ContainsRealm()    => Assert.Contains(Realm, Cfg(false).ToString());
    [Fact] public void ToString_ContainsDeviceId() => Assert.Contains(DevId, Cfg(false).ToString());
}
