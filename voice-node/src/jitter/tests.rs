use super::*;

fn pkt(seq: u32, arrived_at: u64) -> BufferedPacket {
    BufferedPacket {
        voice_seq: seq,
        timestamp: 0,
        channel_id: 1,
        opus_data: vec![seq as u8],
        arrived_at,
    }
}

#[test]
fn pop_ready_waits_for_target_ms() {
    let mut jb = JitterBuffer::new(20, false);
    jb.push(pkt(1, 100));
    assert!(jb.pop_ready(110).is_none());
    assert_eq!(jb.pop_ready(120).unwrap().voice_seq, 1);
    assert!(jb.pop_ready(120).is_none());
}

#[test]
fn pop_ready_releases_in_sequence_order() {
    let mut jb = JitterBuffer::new(0, false);
    jb.push(pkt(2, 0));
    jb.push(pkt(1, 0));
    assert_eq!(jb.pop_ready(0).unwrap().voice_seq, 1);
    assert_eq!(jb.pop_ready(0).unwrap().voice_seq, 2);
}

#[test]
fn has_gap_after_played_seq() {
    let mut jb = JitterBuffer::new(0, false);
    jb.push(pkt(1, 0));
    jb.pop_ready(0);
    jb.push(pkt(3, 0));
    assert!(jb.has_gap());
}

#[test]
fn no_gap_when_next_seq_present() {
    let mut jb = JitterBuffer::new(0, false);
    jb.push(pkt(1, 0));
    jb.pop_ready(0);
    jb.push(pkt(2, 0));
    assert!(!jb.has_gap());
}

#[test]
fn adaptive_mode_increases_on_loss() {
    let mut jb = JitterBuffer::new(40, true);
    jb.push(pkt(1, 0));
    jb.push(pkt(3, 1));
    for i in 4..=50 {
        jb.push(pkt(i, i as u64));
    }
    assert!(jb.target_ms <= MAX_TARGET_MS);
}

#[test]
fn set_target_ms_clamps() {
    let mut jb = JitterBuffer::new(40, false);
    jb.set_target_ms(0);
    assert_eq!(jb.target_ms, MIN_TARGET_MS);
    jb.set_target_ms(500);
    assert_eq!(jb.target_ms, MAX_TARGET_MS);
}

#[test]
fn adaptive_mode_respects_flag() {
    let mut jb = JitterBuffer::new(40, false);
    jb.push(pkt(1, 0));
    for i in 2..=100 {
        jb.push(pkt(i, i as u64));
    }
    assert_eq!(jb.target_ms, 40);
}

#[test]
fn observed_jitter_smooth_timeline() {
    let mut jb = JitterBuffer::new(40, true);
    for i in 0..JITTER_WINDOW {
        jb.push(pkt(i as u32, (i as u64) * 10));
    }
    let jitter = jb.observed_jitter_ms();
    assert!(
        (8..=12).contains(&jitter),
        "jitter = {jitter} (expected ~10)"
    );
}

#[test]
fn observed_jitter_spike() {
    let mut jb = JitterBuffer::new(40, true);
    let times = [
        0, 10, 20, 70, 80, 90, 140, 150, 160, 210, 220, 230, 280, 290, 300,
    ];
    for (i, &t) in times.iter().enumerate() {
        jb.push(pkt(i as u32, t));
    }
    let jitter = jb.observed_jitter_ms();
    assert!(jitter >= 40, "jitter = {jitter} (expected >= 40)");
}
