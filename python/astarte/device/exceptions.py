"""Custom exceptions for the Astarte Device SDK Python bindings."""

from __future__ import annotations

class InvalidUserData(Exception):
    """Raised when an invalid or unexpected user_data get received from a callback."""
    
    pass

class AstarteError(Exception):
    """Base class for all Astarte SDK errors."""

    pass


class InvalidNativeValueError(AstarteError):
    """Raised when an invalid or unexpected native value/tag is encountered."""

    pass


class GetPropertyError(AstarteError):
    """Raised when getting a property fails."""

    pass


class SendError(AstarteError):
    """Raised when sending data to Astarte fails."""

    pass


class ReceiveError(AstarteError):
    """Raised when receiving data from Astarte fails."""

    pass


class DisconnectError(AstarteError):
    """Raised when disconnecting from Astarte fails."""

    pass


class ConnectError(AstarteError):
    """Raised when connecting to Astarte fails."""

    pass


class HandleEventsError(AstarteError):
    """Raised when the event handle loop encounters an error."""

    pass
