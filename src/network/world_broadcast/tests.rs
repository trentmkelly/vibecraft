use super::*;

fn plain(id: u8, body: &[u8]) -> Vec<u8> {
    let mut p = vec![id];
    p.extend_from_slice(body);
    p
}

fn drain(sub: &Subscription) -> Vec<u8> {
    let mut out = Vec::new();
    sub.drain_into(&mut out, CompressionState::disabled())
        .unwrap();
    out
}

#[test]
fn publish_reaches_every_subscriber_framed_per_connection() {
    let bus = WorldPacketBus::default();
    let a = bus.subscribe(1);
    let b = bus.subscribe(2);
    bus.publish(&plain(0x05, &[9, 9]));
    // Disabled compression frame = VarInt length + payload.
    assert_eq!(drain(&a), vec![3, 0x05, 9, 9]);
    assert_eq!(drain(&b), vec![3, 0x05, 9, 9]);
    assert!(drain(&a).is_empty(), "inbox is emptied by draining");
}

#[test]
fn dropping_a_subscription_unsubscribes_it() {
    let bus = WorldPacketBus::default();
    drop(bus.subscribe(1));
    bus.publish(&plain(1, &[]));
    assert!(bus.lock().is_empty());
}

#[test]
fn publish_frames_splits_uncompressed_frames() {
    let bus = WorldPacketBus::default();
    let a = bus.subscribe(1);
    bus.publish_frames(&[2, 0x01, 0xAA, 1, 0x02]).unwrap();
    assert_eq!(drain(&a), vec![2, 0x01, 0xAA, 1, 0x02]);
}

#[test]
fn publish_frames_rejects_truncated_input() {
    let bus = WorldPacketBus::default();
    assert!(bus.publish_frames(&[5, 0x01]).is_err());
}

#[test]
fn drain_applies_the_connection_compression_state() {
    let bus = WorldPacketBus::default();
    let a = bus.subscribe(1);
    bus.publish(&plain(0x01, &[0; 4]));
    let mut out = Vec::new();
    a.drain_into(&mut out, CompressionState::enabled(256)).unwrap();
    // Below threshold: frame length, then a zero "uncompressed" marker.
    assert_eq!(out, vec![6, 0, 0x01, 0, 0, 0, 0]);
}

#[test]
fn publish_to_and_except_target_individual_connections() {
    let bus = WorldPacketBus::default();
    let a = bus.subscribe(1);
    let b = bus.subscribe(2);
    let c = bus.subscribe(3);
    assert!(bus.publish_to(2, &plain(0x01, &[])));
    assert!(!bus.publish_to(99, &plain(0x01, &[])), "unknown token");
    bus.publish_except(2, &plain(0x02, &[]));
    assert_eq!(drain(&a), vec![1, 0x02]);
    assert_eq!(drain(&b), vec![1, 0x01]);
    assert_eq!(drain(&c), vec![1, 0x02]);
}

#[test]
fn disconnect_queues_the_notice_latches_closing_and_drops_later_packets() {
    let bus = WorldPacketBus::default();
    let a = bus.subscribe(1);
    bus.publish(&plain(0x01, &[]));
    assert!(bus.disconnect(1, &plain(0x09, &[7])));
    assert!(!bus.publish_to(1, &plain(0x03, &[])), "closing inbox refuses packets");
    bus.publish(&plain(0x04, &[]));
    assert!(!bus.disconnect(1, &plain(0x09, &[])), "already closing");
    assert!(a.is_closing());
    assert_eq!(drain(&a), vec![1, 0x01, 2, 0x09, 7]);
    assert!(a.is_closing(), "the latch survives draining");
}
