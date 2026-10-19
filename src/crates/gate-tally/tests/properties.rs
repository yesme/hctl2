//! Property cases over generated fact sequences. Facts are recorded one at a
//! time and recounted after each, as control does. Whether a round passes is
//! decided by the first conclusion that is not `Pending`; that conclusion does
//! not depend on the order facts arrive in, and neither a pass nor a no-pass
//! turns into anything else afterwards.
//!
//! The generator is a fixed-seed splitmix64, so every failure names the seed
//! that reproduces it; no third-party crate is needed for that.

use gate_tally::*;

const WORLDS: u64 = 3000;
const ORDERS: usize = 8;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i + 1));
        }
    }
}

/// What can be recorded about one seat, in the order it can happen.
#[derive(Clone, Debug)]
enum Fact {
    Dispatched,
    Delivered(Delivery),
    Failed { allowed: bool, profile: Profile },
    Proposed(Key),
    Admitted(Verdict),
    DeadlinePassed,
}

fn record(seat: &mut Seat, fact: &Fact) {
    match fact {
        Fact::Dispatched => seat.attempt = Attempt::Running,
        Fact::Delivered(delivery) => seat.delivered = Some(delivery.clone()),
        Fact::Failed { allowed, profile } => {
            seat.attempt = Attempt::Failed { allowed: *allowed };
            if *allowed {
                seat.candidates.remove(profile);
            }
        }
        Fact::Proposed(key) => {
            seat.attempt = Attempt::Proposed;
            seat.pending = Some(key.clone());
        }
        Fact::Admitted(verdict) => {
            let key = seat.pending.take().expect("admission follows a proposal");
            seat.vote = Some(Vote {
                verdict: *verdict,
                key,
                evidence_channel: "adapter_event".into(),
            });
        }
        Fact::DeadlinePassed => seat.end = Some(End::DeadlinePassed),
    }
}

struct World {
    quorum: usize,
    seats: Vec<Seat>,
    facts: Vec<Vec<Fact>>,
}

/// Up to four seats over three profiles and two materials. A delivery digest
/// carries its material, so seats whose materials differ never deliver alike.
fn world(rng: &mut Rng) -> World {
    let profiles = ["p0", "p1", "p2"];
    let n = 1 + rng.below(4);
    let quorum = 1 + rng.below(n);
    let mut seats = Vec::new();
    let mut facts = Vec::new();
    for i in 0..n {
        let material = format!("m{}", rng.below(2));
        let mask = 1 + rng.below(7);
        let mut candidates: Vec<&str> = (0..3)
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| profiles[bit])
            .collect();
        let mut seat = Seat::new(&format!("s{i}"), &material, &candidates);
        seat.veto = rng.below(5) == 0;
        let delivery = format!("{material}/{}", rng.below(2));
        let mut sequence = vec![Fact::Dispatched, Fact::Delivered(delivery.clone())];
        rng.shuffle(&mut candidates);
        for (tried, profile) in candidates.iter().enumerate() {
            match rng.below(10) {
                0..=5 => {
                    sequence.push(Fact::Proposed(Key {
                        profile: (*profile).into(),
                        delivery: delivery.clone(),
                    }));
                    sequence.push(Fact::Admitted(match rng.below(5) {
                        0 | 1 => Verdict::Accepted,
                        2 => Verdict::ChangesRequested(Locus::Implementation),
                        3 => Verdict::ChangesRequested(Locus::Contract),
                        _ => Verdict::Rejected,
                    }));
                    break;
                }
                6 | 7 => {
                    sequence.push(Fact::Failed {
                        allowed: true,
                        profile: (*profile).into(),
                    });
                    if tried + 1 < candidates.len() {
                        sequence.push(Fact::Dispatched);
                    }
                }
                8 => {
                    sequence.push(Fact::Failed {
                        allowed: false,
                        profile: (*profile).into(),
                    });
                    break;
                }
                _ => {
                    sequence.push(Fact::DeadlinePassed);
                    break;
                }
            }
        }
        seats.push(seat);
        facts.push(sequence);
    }
    World {
        quorum,
        seats,
        facts,
    }
}

fn count(quorum: usize, seats: &[Seat]) -> Tally {
    tally(&Gate {
        quorum,
        seats: seats.to_vec(),
    })
    .unwrap()
}

/// The first conclusion that was not `Pending`, and what step three held then.
struct First {
    passed: bool,
    findings: Findings,
}

struct Run {
    first: First,
    last: Tally,
}

/// Records facts one at a time, recounting after each. A pass revokes the
/// listed seats; whatever they hand in later is history and never recorded.
/// After a no-pass the remaining facts are still recorded, only to check that
/// none of them turns it back: what control does with the round then is the
/// reducer's, not the count's.
fn run(world: &World, order: &[usize]) -> Result<Run, String> {
    let mut seats = world.seats.clone();
    let mut next = vec![0; seats.len()];
    let mut signed: Option<Vec<Key>> = None;
    let mut first: Option<First> = None;
    for &i in order {
        let fact = &world.facts[i][next[i]];
        next[i] += 1;
        if seats[i].end == Some(End::Cancelled) {
            continue;
        }
        record(&mut seats[i], fact);
        let tally = count(world.quorum, &seats);
        let no_pass = first.as_ref().is_some_and(|first| !first.passed);
        match (&tally.outcome, &signed) {
            (Outcome::Pass { counted, .. }, Some(before)) => {
                if !before.iter().all(|key| counted.contains(key)) {
                    return Err(format!("signed groups {before:?} dropped: {counted:?}"));
                }
            }
            (other, Some(_)) => return Err(format!("a signed pass became {other:?}")),
            (Outcome::Pass { .. }, None) if no_pass => {
                return Err("a no-pass became a pass".into());
            }
            (Outcome::Pass { counted, revoke }, None) => {
                signed = Some(counted.clone());
                first = Some(First {
                    passed: true,
                    findings: tally.findings.clone(),
                });
                for seat in seats.iter_mut().filter(|seat| revoke.contains(&seat.id)) {
                    seat.end = Some(End::Cancelled);
                }
            }
            (Outcome::NoPass { .. }, None) => {
                first.get_or_insert_with(|| First {
                    passed: false,
                    findings: tally.findings.clone(),
                });
            }
            (Outcome::Pending { .. }, None) if no_pass => {
                return Err(format!("a no-pass became pending: {tally:?}"));
            }
            (Outcome::Pending { .. }, None) => {}
        }
    }
    let last = count(world.quorum, &seats);
    if matches!(last.outcome, Outcome::Pending { .. }) {
        return Err(format!("still pending once every seat ended: {last:?}"));
    }
    let first = first.ok_or("no conclusion was ever reached")?;
    Ok(Run { first, last })
}

#[test]
fn pass_or_no_pass_does_not_depend_on_arrival_order_and_never_turns() {
    let mut passes = 0;
    let mut findings_differ = 0;
    for seed in 0..WORLDS {
        let mut rng = Rng(seed);
        let world = world(&mut rng);
        let mut order: Vec<usize> = world
            .facts
            .iter()
            .enumerate()
            .flat_map(|(i, facts)| std::iter::repeat_n(i, facts.len()))
            .collect();
        let mut runs = Vec::new();
        for _ in 0..ORDERS {
            rng.shuffle(&mut order);
            runs.push(run(&world, &order).unwrap_or_else(|e| panic!("seed {seed}: {e}")));
        }
        let first = &runs[0];
        for other in &runs[1..] {
            assert_eq!(
                first.first.passed, other.first.passed,
                "seed {seed}: pass depends on order"
            );
            if !first.first.passed {
                // Nothing was revoked, so every order recorded the same facts.
                assert_eq!(first.last, other.last, "seed {seed}");
            }
        }
        passes += usize::from(first.first.passed);
        // Not asserted: when the round first cannot pass, what step three holds
        // may still depend on order. When to act on it is the reducer's.
        findings_differ += usize::from(
            !first.first.passed
                && runs[1..]
                    .iter()
                    .any(|other| other.first.findings != first.first.findings),
        );
    }
    eprintln!("{passes} of {WORLDS} worlds passed");
    eprintln!(
        "{findings_differ} of the {} that did not pass held different step-three findings at their first no-pass, depending on order",
        WORLDS as usize - passes
    );
    // The generator has to reach both conclusions for the property to mean anything.
    assert!(
        passes > 0 && passes < WORLDS as usize,
        "{passes} of {WORLDS} passed"
    );
}

#[test]
fn seat_order_in_the_input_changes_nothing() {
    for seed in 0..WORLDS {
        let mut rng = Rng(seed);
        let world = world(&mut rng);
        let mut seats = world.seats.clone();
        for (seat, facts) in seats.iter_mut().zip(&world.facts) {
            let upto = rng.below(facts.len() + 1);
            for fact in &facts[..upto] {
                record(seat, fact);
            }
        }
        let expected = count(world.quorum, &seats);
        rng.shuffle(&mut seats);
        assert_eq!(count(world.quorum, &seats), expected, "seed {seed}");
    }
}
