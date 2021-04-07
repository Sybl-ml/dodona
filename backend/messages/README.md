# `messages`

Various message types that can be sent between components. Messages are
typically sent using length-prefixing, with this library containing a generic
implementation for anything that can be serialized using the
[`serde`](https://serde.rs/) library.

This is where the DCL-client messages are defined along with their structure
and expected types.
