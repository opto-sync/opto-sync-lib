use opto_sync_lib::scoped_cursor::{
    can_apply_change, can_resume, can_rotate, ATTO_SIGNED_MAX,
};

#[test]
fn bounded_scoped_change_cases_require_exact_cursor_and_verified_scope() {
    let mut admitted = 0_usize;
    let mut denied = 0_usize;
    for active in 0..=3 {
        for observed in 0..=3 {
            for local in 0..=3 {
                for previous in 0..=3 {
                    for next in 0..=5 {
                        for scope in [false, true] {
                            for trusted in [false, true] {
                                let valid = can_apply_change(
                                    active, observed, local, previous, next, scope, trusted,
                                );
                                if valid {
                                    admitted += 1;
                                    assert!(active > 0 && scope && trusted);
                                    assert_eq!(active, observed);
                                    assert_eq!(previous, local);
                                    assert!(next > local);
                                } else {
                                    denied += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(admitted, 42);
    assert!(denied > admitted);
    assert!(can_apply_change(1, 1, 2, 2, 4, true, true));
    assert!(!can_apply_change(1, 1, 2, 1, 4, true, true));
    assert!(!can_apply_change(1, 1, 2, 2, 2, true, true));
    assert!(!can_apply_change(1, 2, 2, 2, 4, true, true));
    assert!(!can_apply_change(1, 1, 2, 2, 4, false, true));
}

#[test]
fn resume_and_rotation_require_committed_scoped_evidence() {
    assert!(can_resume(2, 2, 10, 10, true, true));
    assert!(!can_resume(2, 1, 10, 10, true, true));
    assert!(!can_resume(2, 2, 10, 11, true, true));
    assert!(!can_resume(2, 2, 10, 10, false, true));
    assert!(!can_resume(2, 2, 10, 10, true, false));
    assert!(can_rotate(2, 3, 0, true, true));
    assert!(!can_rotate(2, 2, 0, true, true));
    assert!(!can_rotate(2, 3, 1, true, true));
    assert!(!can_rotate(2, 3, 0, false, true));
    assert!(!can_rotate(2, 3, 0, true, false));
}

#[test]
fn signed_i64_edges_never_wrap_or_promote_impossible_generation() {
    assert!(can_apply_change(
        1, 1, ATTO_SIGNED_MAX - 1, ATTO_SIGNED_MAX - 1, ATTO_SIGNED_MAX, true, true,
    ));
    assert!(!can_apply_change(
        1, 1, ATTO_SIGNED_MAX, ATTO_SIGNED_MAX, ATTO_SIGNED_MAX + 1, true, true,
    ));
    assert!(!can_resume(
        ATTO_SIGNED_MAX + 1, ATTO_SIGNED_MAX + 1, 0, 0, true, true,
    ));
    assert!(!can_rotate(
        ATTO_SIGNED_MAX, ATTO_SIGNED_MAX + 1, 0, true, true,
    ));
}
