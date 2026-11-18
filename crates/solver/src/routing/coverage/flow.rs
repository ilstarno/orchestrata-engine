use super::{FlowWitness, budget::Budget};

/// Checks the finite graph, matching, cut closure and equality independently of
/// the augmenting-path algorithm. This validates the graph witness, not the
/// routing relaxation used to construct that graph.
pub fn verify_flow_witness(w: &FlowWitness) -> bool {
    let n = w.eligible_slots.len();
    let m = w.slot_capacities.len();
    if n > 32
        || m > 512
        || w.matching.len() != n
        || w.source_jobs.len() != n
        || w.source_slots.len() != m
        || w.slot_capacities.iter().any(|&c| c > n)
    {
        return false;
    }
    let mut loads = vec![0; m];
    for (job, edges) in w.eligible_slots.iter().enumerate() {
        if edges.len() > 16
            || edges.iter().enumerate().any(|(i, &s)| {
                s >= m || edges[..i].contains(&s) || (w.source_jobs[job] && !w.source_slots[s])
            })
        {
            return false;
        }
        if let Some(s) = w.matching[job] {
            if !edges.contains(&s) {
                return false;
            }
            loads[s] += 1;
        }
    }
    if loads.iter().zip(&w.slot_capacities).any(|(a, b)| a > b) {
        return false;
    }
    let cut = w.source_jobs.iter().filter(|&&x| !x).count()
        + w.slot_capacities
            .iter()
            .zip(&w.source_slots)
            .filter_map(|(&c, &s)| s.then_some(c))
            .sum::<usize>();
    cut == w.cut_capacity && cut == w.matching.iter().flatten().count()
}

pub(super) fn solve(
    edges: Vec<Vec<usize>>,
    caps: Vec<usize>,
    b: &mut Budget,
) -> Result<FlowWitness, ()> {
    let n = edges.len();
    let m = caps.len();
    let mut matching = vec![None; n];
    let mut owners: Vec<Vec<usize>> = vec![Vec::new(); m];
    for root in 0..n {
        let mut seen_jobs = vec![false; n];
        let mut parent = vec![None; m];
        seen_jobs[root] = true;
        let mut queue = vec![root];
        let mut next = 0;
        let mut free = None;
        'search: while next < queue.len() {
            let job = queue[next];
            next += 1;
            for &slot in &edges[job] {
                b.step()?;
                if parent[slot].is_some() {
                    continue;
                }
                parent[slot] = Some(job);
                if owners[slot].len() < caps[slot] {
                    free = Some(slot);
                    break 'search;
                }
                for &other in &owners[slot] {
                    b.step()?;
                    if !seen_jobs[other] {
                        seen_jobs[other] = true;
                        queue.push(other);
                    }
                }
            }
        }
        while let Some(slot) = free {
            b.step()?;
            let job = parent[slot].ok_or(())?;
            free = matching[job];
            if let Some(old) = free {
                owners[old].retain(|&j| j != job);
            }
            matching[job] = Some(slot);
            owners[slot].push(job);
        }
    }
    let mut source_jobs: Vec<_> = matching.iter().map(Option::is_none).collect();
    let mut source_slots = vec![false; m];
    let mut queue: Vec<_> = (0..n).filter(|&j| source_jobs[j]).collect();
    let mut next = 0;
    while next < queue.len() {
        let job = queue[next];
        next += 1;
        for &slot in &edges[job] {
            b.step()?;
            if std::mem::replace(&mut source_slots[slot], true) {
                continue;
            }
            for &other in &owners[slot] {
                b.step()?;
                if !std::mem::replace(&mut source_jobs[other], true) {
                    queue.push(other);
                }
            }
        }
    }
    let cut_capacity = matching.iter().flatten().count();
    Ok(FlowWitness {
        eligible_slots: edges,
        slot_capacities: caps,
        matching,
        source_jobs,
        source_slots,
        cut_capacity,
    })
}
