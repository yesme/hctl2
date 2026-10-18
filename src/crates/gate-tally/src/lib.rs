//! The vote count of a review Gate (`docs/design/spec/run.md` §计票与结束).
//!
//! Input is recorded facts and frozen data; output is pass, quorum unreachable
//! or not yet collected, plus the seats to revoke. Nothing here reads or writes
//! storage, and nothing depends on the order facts arrived in.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

/// A Worker Profile digest: harness, model, mode and permissions together.
pub type Profile = String;

/// Digest of what a seat's Bundle actually delivered, plus the seat's required
/// Skill references. Consumer identity, metering and retention are not in it.
pub type Delivery = String;

/// Digest of the frozen references a seat's delivery digest is computed from:
/// the ReviewSubjectRef, the seat's required Skill references and the root
/// Manifest. Seats whose materials differ cannot deliver the same content.
pub type Material = String;

/// Two admitted verdicts with the same key count once toward quorum.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    pub profile: Profile,
    pub delivery: Delivery,
}

/// Where a `changes_requested` places the disagreement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locus {
    Implementation,
    Contract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Verdict {
    Accepted,
    ChangesRequested(Locus),
    Rejected,
}

impl Verdict {
    /// Higher is less favourable to passing. A contract disagreement stops
    /// rework and goes to a human, so it ranks above an implementation one.
    fn severity(self) -> u8 {
        match self {
            Verdict::Accepted => 0,
            Verdict::ChangesRequested(Locus::Implementation) => 1,
            Verdict::ChangesRequested(Locus::Contract) => 2,
            Verdict::Rejected => 3,
        }
    }
}

/// A seat's verdict that passed result admission (有效裁决).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vote {
    pub verdict: Verdict,
    pub key: Key,
    /// Recorded for the Gate Receipt. Never part of the key (CT-RUN 32).
    pub evidence_channel: String,
}

/// The seat's latest Attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attempt {
    NotDispatched,
    Running,
    /// Ended by handing in a proposal. This ends the Attempt, not the seat.
    Proposed,
    /// Failed, lost or revoked. `allowed` is whether the frozen policy lists
    /// the fault as one that may open another Attempt.
    Failed {
        allowed: bool,
    },
}

/// Ends decided by the existing admission and revocation rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Cancelled,
    Superseded,
    DeadlinePassed,
}

/// One Gate Seat: its frozen data and the facts recorded for it so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    pub id: String,
    /// The Gate policy declares a veto for this seat.
    pub veto: bool,
    /// Profiles the seat may still produce a verdict under: the running
    /// Attempt's and the approved candidates not yet tried and failed.
    pub candidates: BTreeSet<Profile>,
    pub material: Material,
    /// Set once the Bundle is delivered.
    pub delivered: Option<Delivery>,
    pub attempt: Attempt,
    /// A frozen proposal awaiting admission, under the key it was produced with.
    /// It is neither counted nor treated as absent.
    pub pending: Option<Key>,
    pub vote: Option<Vote>,
    pub end: Option<End>,
    /// The budget still allows a new Attempt.
    pub budget_left: bool,
    /// The remaining deadline still allows a new Attempt.
    pub time_left: bool,
}

impl Seat {
    /// A seat not yet dispatched, with budget and time left.
    pub fn new(id: &str, material: &str, candidates: &[&str]) -> Self {
        Self {
            id: id.into(),
            veto: false,
            candidates: candidates.iter().map(|p| (*p).into()).collect(),
            material: material.into(),
            delivered: None,
            attempt: Attempt::NotDispatched,
            pending: None,
            vote: None,
            end: None,
            budget_left: true,
            time_left: true,
        }
    }

    /// Whether control may open an Attempt for this seat now: the first one,
    /// or a backup after an allowed fault. A seat that handed in a proposal
    /// does not switch, and a cancelled or superseded one is not dispatched.
    pub fn can_open_attempt(&self) -> bool {
        self.end.is_none()
            && self.vote.is_none()
            && self.pending.is_none()
            && matches!(
                self.attempt,
                Attempt::NotDispatched | Attempt::Failed { allowed: true }
            )
            && !self.candidates.is_empty()
            && self.budget_left
            && self.time_left
    }

    /// 仍可产出: the obligation is valid and before its deadline, there is no
    /// admitted verdict yet, and a proposal awaits admission, an Attempt runs,
    /// or another Attempt may be opened.
    pub fn producing(&self) -> bool {
        self.end.is_none()
            && self.vote.is_none()
            && (self.pending.is_some()
                || self.attempt == Attempt::Running
                || self.can_open_attempt())
    }

    /// 已结束: an admitted verdict, the deadline, exhausted candidates,
    /// cancellation or supersession.
    pub fn ended(&self) -> bool {
        !self.producing()
    }

    /// 仍可能加入 the group of `key`, whose members froze `materials`: the seat
    /// is still producing and nothing recorded proves its key differs.
    pub fn may_join(&self, key: &Key, materials: &BTreeSet<Material>) -> bool {
        if !self.producing() {
            return false;
        }
        if let Some(pending) = &self.pending {
            return pending == key;
        }
        self.candidates.contains(&key.profile)
            && match &self.delivered {
                Some(delivery) => delivery == &key.delivery,
                None => materials.contains(&self.material),
            }
    }
}

/// The frozen quorum and every Seat of one Gate round.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    pub quorum: usize,
    pub seats: Vec<Seat>,
}

/// Admitted verdicts that share a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub key: Key,
    /// Seat ids, sorted.
    pub seats: Vec<String>,
    /// The least favourable verdict in the group.
    pub verdict: Verdict,
    /// The verdicts disagree: the group counts as `verdict` and needs attention.
    pub split: bool,
    /// 已定: no seat may still join.
    pub decided: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Sign the summary Verdict/Receipt, then revoke `revoke`. Proposals those
    /// seats hand in later stay history; they form no group and change nothing.
    Pass {
        counted: Vec<Key>,
        revoke: Vec<String>,
    },
    /// A veto seat's admitted verdict is not `accepted`: the rework path.
    Vetoed { seats: Vec<String> },
    /// A veto seat ended without an admitted verdict. Candidates are exhausted:
    /// a Request or a typed technical failure, never a pass.
    VetoUnresolved { seats: Vec<String> },
    /// Even the most favourable landing of every producing seat falls short
    /// (`quorum-unreachable`).
    QuorumUnreachable { upper_bound: usize },
    /// Not yet collected. Each seat's ending bounds the wait.
    Pending { upper_bound: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tally {
    pub outcome: Outcome,
    /// Sorted by key.
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invalid {
    ZeroQuorum,
    DuplicateSeat(String),
}

/// Recounts from recorded facts only (§计票与结束, steps two to five; step one,
/// result admission, happened before a vote is recorded on a seat).
pub fn tally(gate: &Gate) -> Result<Tally, Invalid> {
    if gate.quorum == 0 {
        return Err(Invalid::ZeroQuorum);
    }
    let mut seats: Vec<&Seat> = gate.seats.iter().collect();
    seats.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(pair) = seats.windows(2).find(|pair| pair[0].id == pair[1].id) {
        return Err(Invalid::DuplicateSeat(pair[0].id.clone()));
    }
    let groups = groups(&seats);
    let outcome = outcome(gate.quorum, &seats, &groups);
    Ok(Tally { outcome, groups })
}

fn groups(seats: &[&Seat]) -> Vec<Group> {
    let mut by_key: BTreeMap<&Key, Vec<&Seat>> = BTreeMap::new();
    for seat in seats {
        if let Some(vote) = &seat.vote {
            by_key.entry(&vote.key).or_default().push(seat);
        }
    }
    by_key
        .into_iter()
        .map(|(key, members)| {
            let verdicts: Vec<Verdict> = members
                .iter()
                .filter_map(|seat| seat.vote.as_ref().map(|vote| vote.verdict))
                .collect();
            let verdict = verdicts
                .iter()
                .copied()
                .max_by_key(|verdict| verdict.severity())
                .expect("a group has a vote");
            let materials: BTreeSet<Material> =
                members.iter().map(|seat| seat.material.clone()).collect();
            Group {
                key: key.clone(),
                seats: members.iter().map(|seat| seat.id.clone()).collect(),
                verdict,
                split: verdicts.iter().any(|other| *other != verdict),
                decided: !seats.iter().any(|seat| seat.may_join(key, &materials)),
            }
        })
        .collect()
}

fn outcome(quorum: usize, seats: &[&Seat], groups: &[Group]) -> Outcome {
    // Step three: a veto decides before anything is counted.
    let vetoed = ids(seats, |seat| {
        seat.veto
            && seat
                .vote
                .as_ref()
                .is_some_and(|vote| vote.verdict != Verdict::Accepted)
    });
    if !vetoed.is_empty() {
        return Outcome::Vetoed { seats: vetoed };
    }
    let unresolved = ids(seats, |seat| {
        seat.veto && seat.vote.is_none() && seat.ended()
    });
    if !unresolved.is_empty() {
        return Outcome::VetoUnresolved { seats: unresolved };
    }
    // Step four: enough decided passing groups, and no veto still to come.
    let counted: Vec<Key> = groups
        .iter()
        .filter(|group| group.decided && group.verdict == Verdict::Accepted)
        .map(|group| group.key.clone())
        .collect();
    let veto_open = seats.iter().any(|seat| seat.veto && seat.producing());
    if counted.len() >= quorum && !veto_open {
        let revoke = ids(seats, |seat| seat.producing() && seat.pending.is_none());
        return Outcome::Pass { counted, revoke };
    }
    // Step five: the most passing groups still possible.
    let upper_bound = upper_bound(seats, groups);
    if upper_bound < quorum {
        Outcome::QuorumUnreachable { upper_bound }
    } else {
        Outcome::Pending { upper_bound }
    }
}

fn ids(seats: &[&Seat], keep: impl Fn(&Seat) -> bool) -> Vec<String> {
    seats
        .iter()
        .filter(|seat| keep(seat))
        .map(|seat| seat.id.clone())
        .collect()
}

/// Every admitted verdict, decided group or not, is a fixed input. Each
/// producing seat then lands where it helps most: accepted, under a key no
/// other group holds. A seat whose Bundle is not delivered can always land on
/// a key of its own, since nothing proves its content equal to another's.
fn upper_bound(seats: &[&Seat], groups: &[Group]) -> usize {
    let taken: BTreeSet<&Key> = groups.iter().map(|group| &group.key).collect();
    let passing = groups
        .iter()
        .filter(|group| group.verdict == Verdict::Accepted)
        .count();
    let mut open = 0;
    let mut options: Vec<Vec<Key>> = Vec::new();
    for seat in seats.iter().filter(|seat| seat.producing()) {
        let keys: Vec<Key> = match (&seat.pending, &seat.delivered) {
            (Some(key), _) => vec![key.clone()],
            (None, Some(delivery)) => seat
                .candidates
                .iter()
                .map(|profile| Key {
                    profile: profile.clone(),
                    delivery: delivery.clone(),
                })
                .collect(),
            (None, None) => {
                open += 1;
                continue;
            }
        };
        options.push(
            keys.into_iter()
                .filter(|key| !taken.contains(key))
                .collect(),
        );
    }
    passing + open + max_matching(&options)
}

/// The most seats that can each take a different key of their own (Kuhn's
/// augmenting paths; a Gate has a handful of seats).
fn max_matching(options: &[Vec<Key>]) -> usize {
    let mut owner: BTreeMap<&Key, usize> = BTreeMap::new();
    (0..options.len())
        .filter(|&seat| augment(seat, options, &mut owner, &mut BTreeSet::new()))
        .count()
}

fn augment<'a>(
    seat: usize,
    options: &'a [Vec<Key>],
    owner: &mut BTreeMap<&'a Key, usize>,
    seen: &mut BTreeSet<&'a Key>,
) -> bool {
    for key in &options[seat] {
        if !seen.insert(key) {
            continue;
        }
        let free = match owner.get(key).copied() {
            None => true,
            Some(other) => augment(other, options, owner, seen),
        };
        if free {
            owner.insert(key, seat);
            return true;
        }
    }
    false
}

/// The two seat-diversity items a Gate policy may declare. Evidence channel
/// and harness are not among them (CT-RUN 32).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Diversity {
    pub distinct_profiles: bool,
    pub avoid_author_profile: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    SameProfile {
        seats: [String; 2],
        profile: Profile,
    },
    AuthorProfile {
        seat: String,
    },
}

impl Diversity {
    /// Checks the profiles seats are selected with at start, or run with after
    /// a switch. Any violation rejects the start or the switch.
    pub fn check(&self, author: Option<&Profile>, seats: &[(String, Profile)]) -> Vec<Violation> {
        let mut seats = seats.to_vec();
        seats.sort();
        let mut violations = Vec::new();
        if self.distinct_profiles {
            for (i, (a, profile)) in seats.iter().enumerate() {
                for (b, other) in &seats[i + 1..] {
                    if profile == other {
                        violations.push(Violation::SameProfile {
                            seats: [a.clone(), b.clone()],
                            profile: profile.clone(),
                        });
                    }
                }
            }
        }
        if self.avoid_author_profile
            && let Some(author) = author
        {
            for (seat, profile) in &seats {
                if profile == author {
                    violations.push(Violation::AuthorProfile { seat: seat.clone() });
                }
            }
        }
        violations
    }
}
