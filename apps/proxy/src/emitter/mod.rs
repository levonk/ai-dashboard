//! Emitter mode implementation
//! 
//! This module provides the foundation for enterprise emitter mode in the proxy service.
//! Emitter mode enables high-scale deployments by separating data collection from local
//! processing, allowing the proxy to emit analytics events to external collectors and
//! analytics services.

pub mod protocol;

pub use protocol::{
    ProtocolConfig,
    EventSerializer,
    EventDeserializer,
    SerializationFormat,
    ProtocolHeader,
    EventMessage,
    TransmissionMetadata,
    PROTOCOL_VERSION,
};
