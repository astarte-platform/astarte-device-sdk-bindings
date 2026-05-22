using AstarteDeviceSdkBindings;
using Xunit;

namespace AstarteSdk.Tests;

public class ObjectEntryTests
{
    private static ObjectEntry E(string key, AstarteVal value) => new ObjectEntry(key, value);

    [Fact] public void Fields_AreAccessible()
    {
        var e = E("temperature", new AstarteVal.Double(23.5));
        Assert.Equal("temperature", e.Key);
        Assert.IsType<AstarteVal.Double>(e.Value);
    }

    [Fact] public void Equals_SameInstance() { var e = E("k", new AstarteVal.Integer(1)); Assert.Equal(e, e); }

    [Fact] public void Equals_TwoEqualInstances() =>
        Assert.Equal(E("k", new AstarteVal.Integer(1)), E("k", new AstarteVal.Integer(1)));

    [Fact] public void NotEquals_DifferentKey() =>
        Assert.NotEqual(E("a", new AstarteVal.Integer(1)), E("b", new AstarteVal.Integer(1)));

    [Fact] public void NotEquals_DifferentValue() =>
        Assert.NotEqual(E("k", new AstarteVal.Integer(1)), E("k", new AstarteVal.Integer(2)));

    [Fact] public void NotEquals_Null() => Assert.False(E("k", new AstarteVal.Integer(0)).Equals((object?)null));

    [Fact] public void Hashcode_ConsistentWithEquals()
    {
        var a = E("key", new AstarteVal.IString("hello"));
        var b = E("key", new AstarteVal.IString("hello"));
        Assert.Equal(a, b);
        Assert.Equal(a.GetHashCode(), b.GetHashCode());
    }

    [Fact] public void ToString_ContainsKeyAndValue()
    {
        string s = E("sensor", new AstarteVal.Boolean(true)).ToString();
        Assert.Contains("sensor", s);
        Assert.True(s.Contains("Boolean") || s.Contains("True") || s.Contains("true"));
    }
}
