/// Services module - Business logic and analytics services
pub mod meilisearch_client;
pub mod vip3_analytics;
pub mod zeromq_broker;

pub use meilisearch_client::{MeilisearchClient, MidiSearchDocument, SearchFilters, SearchResult};
pub use vip3_analytics::{FilterCounts, VIP3AnalyticsService};
pub use zeromq_broker::{
    BpmMessage, BpmSource, ClockTickMessage, DeviceEvent, DiscoveryRequest, DiscoveryResponse,
    HardwareDevice, HardwareDeviceType, HardwareSyncBridge, SyncControlRequest,
    SyncControlResponse, SyncStatusMessage, TransportMessage, ZmqBroker, ZmqBrokerConfig,
    ZmqClient,
};
