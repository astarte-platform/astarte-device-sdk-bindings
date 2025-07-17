# Why we should switch to the rust binded python sdk ?

- one example is the handling of properties
in the python sdk properties are deleted when unset
this means that unset updates while offline are not sent
is this a problem? is this real ? have to check it better before relying on this point as the base for my thesys

- i think there are many other instances where the handling of edgecases is better
in the rust or zephyr sdk
