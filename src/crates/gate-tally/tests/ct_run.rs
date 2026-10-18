//! CT-RUN 29–32, sentence by sentence, plus the tally half of 17 and 31.

use gate_tally::*;

const P: &str = "claude-code/opus";
const P2: &str = "codex/gpt";
const P3: &str = "grok/build";

fn key(profile: &str, delivery: &str) -> Key {
    Key {
        profile: profile.into(),
        delivery: delivery.into(),
    }
}

fn seat(id: &str, material: &str, candidates: &[&str]) -> Seat {
    Seat::new(id, material, candidates)
}

fn running(mut seat: Seat, delivered: Option<&str>) -> Seat {
    seat.attempt = Attempt::Running;
    seat.delivered = delivered.map(Into::into);
    seat
}

fn voted(mut seat: Seat, verdict: Verdict, profile: &str, delivery: &str) -> Seat {
    seat.attempt = Attempt::Proposed;
    seat.delivered = Some(delivery.into());
    seat.vote = Some(Vote {
        verdict,
        key: key(profile, delivery),
        evidence_channel: "adapter_event".into(),
    });
    seat
}

fn veto(mut seat: Seat) -> Seat {
    seat.veto = true;
    seat
}

fn count(quorum: usize, seats: &[Seat]) -> Tally {
    tally(&Gate {
        quorum,
        seats: seats.to_vec(),
    })
    .unwrap()
}

fn outcome(quorum: usize, seats: &[Seat]) -> Outcome {
    count(quorum, seats).outcome
}

const CR: Verdict = Verdict::ChangesRequested(Locus::Implementation);

// --- CT-RUN 29: duplicate groups ---

#[test]
fn quorum_one_same_group_gives_the_same_result_whichever_verdict_comes_first() {
    let a = || running(seat("a", "m", &[P]), Some("d"));
    let b = || running(seat("b", "m", &[P]), Some("d"));

    // Accepted first: the group is not decided while b may still join it.
    let early = count(1, &[voted(a(), Verdict::Accepted, P, "d"), b()]);
    assert_eq!(early.outcome, Outcome::Pending { upper_bound: 1 });
    assert!(!early.groups[0].decided);
    let accepted_first = count(
        1,
        &[
            voted(a(), Verdict::Accepted, P, "d"),
            voted(b(), CR, P, "d"),
        ],
    );

    // changes_requested first: the group can no longer pass, so the bound is
    // already short (CT-RUN 31); once a's verdict is in, the same group stands.
    assert_eq!(
        outcome(1, &[a(), voted(b(), CR, P, "d")]),
        Outcome::QuorumUnreachable { upper_bound: 0 }
    );
    let requested_first = count(
        1,
        &[
            voted(b(), CR, P, "d"),
            voted(a(), Verdict::Accepted, P, "d"),
        ],
    );

    assert_eq!(accepted_first, requested_first);
    assert_eq!(
        accepted_first.outcome,
        Outcome::QuorumUnreachable { upper_bound: 0 }
    );
    let group = &accepted_first.groups[0];
    assert_eq!(group.seats, ["a", "b"]);
    assert_eq!(group.verdict, CR, "the least favourable verdict stands");
    assert!(group.split && group.decided, "split groups need attention");
}

#[test]
fn quorum_two_with_both_seats_bound_to_one_proven_key_is_unreachable() {
    let a = running(seat("a", "m", &[P]), Some("d"));
    let b = running(seat("b", "m", &[P]), Some("d"));
    assert_eq!(
        outcome(2, &[a.clone(), b.clone()]),
        Outcome::QuorumUnreachable { upper_bound: 1 }
    );

    let both = count(
        2,
        &[
            voted(a, Verdict::Accepted, P, "d"),
            voted(b, Verdict::Accepted, P, "d"),
        ],
    );
    assert_eq!(both.outcome, Outcome::QuorumUnreachable { upper_bound: 1 });
    assert_eq!(both.groups.len(), 1);
    assert_eq!(
        both.groups[0].seats,
        ["a", "b"],
        "both votes stay on record"
    );
}

#[test]
fn a_seat_that_may_still_use_another_key_keeps_quorum_two_open() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let b = running(seat("b", "m", &[P, P2]), Some("d"));
    assert_eq!(outcome(2, &[a, b]), Outcome::Pending { upper_bound: 2 });
}

#[test]
fn an_undispatched_seat_that_may_share_the_key_blocks_pass_and_is_not_revoked() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let same_material = seat("b", "m", &[P]);
    let tally = count(1, &[a.clone(), same_material]);
    assert_eq!(tally.outcome, Outcome::Pending { upper_bound: 2 });
    assert!(!tally.groups[0].decided);

    // Frozen references that differ prove the key differs.
    let other_material = seat("b", "m2", &[P]);
    assert_eq!(
        outcome(1, &[a, other_material]),
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec!["b".into()],
        }
    );
}

#[test]
fn a_frozen_proposal_awaiting_admission_is_neither_counted_nor_revoked() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let mut elsewhere = seat("b", "m2", &[P2]);
    elsewhere.attempt = Attempt::Proposed;
    elsewhere.pending = Some(key(P2, "e"));
    assert_eq!(
        outcome(1, &[a.clone(), elsewhere.clone()]),
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec![],
        }
    );
    assert_eq!(
        outcome(2, &[a.clone(), elsewhere]),
        Outcome::Pending { upper_bound: 2 }
    );

    let mut same_key = seat("b", "m", &[P]);
    same_key.attempt = Attempt::Proposed;
    same_key.pending = Some(key(P, "d"));
    let tally = count(1, &[a, same_key]);
    assert_eq!(tally.outcome, Outcome::Pending { upper_bound: 1 });
    assert_eq!(tally.groups[0].seats, ["a"]);
}

#[test]
fn after_an_early_pass_a_revoked_seats_proposal_stays_history() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let b = running(seat("b", "m2", &[P]), None);
    let signed = outcome(1, &[a.clone(), b.clone()]);
    assert_eq!(
        signed,
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec!["b".into()],
        }
    );

    // Control revokes b; what b hands in afterwards is kept, not admitted.
    let mut revoked = b;
    revoked.end = Some(End::Cancelled);
    revoked.attempt = Attempt::Proposed;
    revoked.pending = Some(key(P, "d"));
    assert_eq!(
        outcome(1, &[a, revoked]),
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec![],
        }
    );
}

#[test]
fn an_ended_seat_no_longer_keeps_a_group_undecided() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let b = seat("b", "m", &[P]);
    assert_eq!(
        outcome(1, &[a.clone(), b.clone()]),
        Outcome::Pending { upper_bound: 2 }
    );
    let pass = Outcome::Pass {
        counted: vec![key(P, "d")],
        revoke: vec![],
    };

    let mut late = b.clone();
    late.end = Some(End::DeadlinePassed);
    assert_eq!(outcome(1, &[a.clone(), late]), pass);

    let mut exhausted = b;
    exhausted.attempt = Attempt::Failed { allowed: false };
    assert_eq!(outcome(1, &[a, exhausted]), pass);
}

#[test]
fn two_ended_seats_without_admitted_verdicts_form_no_vote() {
    let mut a = seat("a", "m", &[P]);
    a.attempt = Attempt::Failed { allowed: false };
    let mut b = seat("b", "m", &[P]);
    b.end = Some(End::Cancelled);
    let tally = count(1, &[a, b]);
    assert!(tally.groups.is_empty());
    assert_eq!(tally.outcome, Outcome::QuorumUnreachable { upper_bound: 0 });
}

// --- CT-RUN 30: switching and veto ---

#[test]
fn without_declared_diversity_a_legal_switch_onto_the_same_key_counts_once() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let mut b = seat("b", "m", &[P]);
    b.delivered = Some("d".into());
    b.attempt = Attempt::Failed { allowed: true };
    assert!(
        b.can_open_attempt(),
        "an allowed fault opens a backup Attempt"
    );

    let switched = voted(b, Verdict::Accepted, P, "d");
    let tally = count(2, &[a.clone(), switched.clone()]);
    assert_eq!(tally.outcome, Outcome::QuorumUnreachable { upper_bound: 1 });
    assert_eq!(tally.groups[0].seats, ["a", "b"]);
    assert_eq!(
        outcome(1, &[a, switched]),
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec![],
        }
    );
}

#[test]
fn a_seat_that_switches_to_a_different_profile_makes_two_votes_and_is_not_written_off_before() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let b = running(seat("b", "m", &[P, P2]), Some("d"));
    assert_eq!(
        outcome(2, &[a.clone(), b.clone()]),
        Outcome::Pending { upper_bound: 2 }
    );

    let mut failed = b;
    failed.attempt = Attempt::Failed { allowed: true };
    failed.candidates.remove(P);
    assert!(failed.can_open_attempt());
    assert_eq!(
        outcome(2, &[a.clone(), failed.clone()]),
        Outcome::Pending { upper_bound: 2 }
    );

    assert_eq!(
        outcome(2, &[a, voted(failed, Verdict::Accepted, P2, "d")]),
        Outcome::Pass {
            counted: vec![key(P, "d"), key(P2, "d")],
            revoke: vec![],
        }
    );
}

#[test]
fn declared_diversity_rejects_a_selection_or_switch_that_breaks_it() {
    let distinct = Diversity {
        distinct_profiles: true,
        avoid_author_profile: false,
    };
    assert_eq!(
        distinct.check(None, &[("b".into(), P.into()), ("a".into(), P.into())]),
        [Violation::SameProfile {
            seats: ["a".into(), "b".into()],
            profile: P.into(),
        }]
    );
    assert!(
        distinct
            .check(None, &[("a".into(), P.into()), ("b".into(), P2.into())])
            .is_empty()
    );

    let avoid = Diversity {
        distinct_profiles: false,
        avoid_author_profile: true,
    };
    let author: Profile = P.into();
    assert_eq!(
        avoid.check(
            Some(&author),
            &[("a".into(), P.into()), ("b".into(), P2.into())]
        ),
        [Violation::AuthorProfile { seat: "a".into() }]
    );
    assert!(
        Diversity::default()
            .check(
                Some(&author),
                &[("a".into(), P.into()), ("b".into(), P.into())]
            )
            .is_empty()
    );
}

#[test]
fn a_producing_veto_seat_without_a_verdict_blocks_pass_and_is_not_revoked() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let v = veto(running(seat("v", "m2", &[P2]), None));
    assert_eq!(
        outcome(1, &[a.clone(), v.clone()]),
        Outcome::Pending { upper_bound: 2 }
    );

    assert_eq!(
        outcome(1, &[a, voted(v, Verdict::Accepted, P2, "e")]),
        Outcome::Pass {
            counted: vec![key(P, "d"), key(P2, "e")],
            revoke: vec![],
        }
    );
}

#[test]
fn a_veto_seat_switches_only_when_the_fault_budget_and_time_allow() {
    let mut v = veto(seat("v", "m", &[P, P2]));
    v.attempt = Attempt::Failed { allowed: true };
    assert!(v.can_open_attempt());

    let not_allowed = Seat {
        attempt: Attempt::Failed { allowed: false },
        ..v.clone()
    };
    let no_budget = Seat {
        budget_left: false,
        ..v.clone()
    };
    let no_time = Seat {
        time_left: false,
        ..v.clone()
    };
    let no_candidates = Seat {
        candidates: Default::default(),
        ..v.clone()
    };
    let proposed = Seat {
        attempt: Attempt::Proposed,
        ..v.clone()
    };
    let cancelled = Seat {
        end: Some(End::Cancelled),
        ..v.clone()
    };
    let superseded = Seat {
        end: Some(End::Superseded),
        ..v
    };
    for seat in [
        not_allowed,
        no_budget,
        no_time,
        no_candidates,
        proposed,
        cancelled,
        superseded,
    ] {
        assert!(!seat.can_open_attempt(), "{seat:?}");
        assert!(seat.ended(), "{seat:?}");
    }
}

#[test]
fn a_veto_seat_that_runs_out_of_candidates_is_a_technical_failure_not_a_pass_or_reject() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let mut v = veto(seat("v", "m2", &[P2]));
    v.attempt = Attempt::Failed { allowed: false };
    assert_eq!(
        outcome(1, &[a, v]),
        Outcome::VetoUnresolved {
            seats: vec!["v".into()],
        }
    );
}

#[test]
fn a_veto_verdict_other_than_accepted_takes_the_rework_path() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let v = veto(voted(seat("v", "m2", &[P2]), Verdict::Rejected, P2, "e"));
    assert_eq!(
        outcome(1, &[a, v]),
        Outcome::Vetoed {
            seats: vec!["v".into()],
        }
    );
}

// --- CT-RUN 31 and 17: the running upper bound ---

#[test]
fn the_upper_bound_counts_admitted_verdicts_in_undecided_groups() {
    // a's rejection sits in a group c may still join, so it is not decided;
    // c can only land on that key, and the rejection already sinks it.
    let a = voted(seat("a", "m", &[P]), Verdict::Rejected, P, "d");
    let c = running(seat("c", "m", &[P]), Some("d"));
    let b = seat("b", "m2", &[P3]);
    let tally = count(2, &[a, b, c]);
    assert!(!tally.groups[0].decided);
    assert_eq!(tally.outcome, Outcome::QuorumUnreachable { upper_bound: 1 });
}

#[test]
fn an_upper_bound_that_still_reaches_quorum_never_fails_early() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let b = running(seat("b", "m", &[P]), None);
    assert_eq!(outcome(2, &[a, b]), Outcome::Pending { upper_bound: 2 });
}

#[test]
fn a_recount_after_restart_does_not_wait_for_an_unrelated_remote_seat() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let remote = running(seat("remote", "m2", &[P]), None);
    assert_eq!(
        outcome(1, &[a, remote]),
        Outcome::Pass {
            counted: vec![key(P, "d")],
            revoke: vec!["remote".into()],
        }
    );
}

#[test]
fn when_every_seat_has_ended_short_of_quorum_the_gate_is_unreachable() {
    let a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let mut b = seat("b", "m2", &[P2]);
    b.attempt = Attempt::Failed { allowed: false };
    assert_eq!(
        outcome(2, &[a, b]),
        Outcome::QuorumUnreachable { upper_bound: 1 }
    );
}

// --- CT-RUN 32: diversity has two items ---

#[test]
fn the_evidence_channel_neither_splits_nor_merges_groups() {
    let mut a = voted(seat("a", "m", &[P]), Verdict::Accepted, P, "d");
    let mut b = voted(seat("b", "m", &[P]), Verdict::Accepted, P, "d");
    a.vote.as_mut().unwrap().evidence_channel = "direct_report".into();
    b.vote.as_mut().unwrap().evidence_channel = "adapter_event".into();
    let tally = count(2, &[a.clone(), b]);
    assert_eq!(tally.groups.len(), 1);
    assert_eq!(tally.outcome, Outcome::QuorumUnreachable { upper_bound: 1 });

    let mut c = voted(seat("c", "m", &[P2]), Verdict::Accepted, P2, "d");
    c.vote.as_mut().unwrap().evidence_channel = "direct_report".into();
    assert_eq!(
        outcome(2, &[a, c]),
        Outcome::Pass {
            counted: vec![key(P, "d"), key(P2, "d")],
            revoke: vec![],
        }
    );
}

#[test]
fn the_same_harness_with_different_profiles_meets_distinct_profiles() {
    let distinct = Diversity {
        distinct_profiles: true,
        avoid_author_profile: true,
    };
    let author: Profile = "codex/gpt".into();
    let seats = [
        ("a".to_string(), "claude-code/opus".to_string()),
        ("b".to_string(), "claude-code/sonnet".to_string()),
    ];
    assert!(distinct.check(Some(&author), &seats).is_empty());
}

#[test]
fn a_restart_behind_the_agency_is_not_a_switch_and_adds_no_vote() {
    // The Attempt keeps running through the restart: one seat, one vote at most.
    let restarted = running(seat("a", "m", &[P, P2]), None);
    let mut gone = seat("b", "m2", &[P]);
    gone.end = Some(End::Cancelled);
    assert_eq!(
        outcome(2, &[restarted, gone]),
        Outcome::QuorumUnreachable { upper_bound: 1 }
    );
}

// --- Input checks ---

#[test]
fn a_zero_quorum_or_a_repeated_seat_is_refused() {
    let a = seat("a", "m", &[P]);
    assert_eq!(
        tally(&Gate {
            quorum: 0,
            seats: vec![a.clone()],
        }),
        Err(Invalid::ZeroQuorum)
    );
    assert_eq!(
        tally(&Gate {
            quorum: 1,
            seats: vec![a.clone(), a],
        }),
        Err(Invalid::DuplicateSeat("a".into()))
    );
}
