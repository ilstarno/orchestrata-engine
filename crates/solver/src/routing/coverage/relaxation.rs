use super::{
    super::{CourierMode, LocationId, RoutingProblem},
    budget::Budget,
};
const INF: u128 = u128::MAX / 4;
const MODES: [CourierMode; 3] = [
    CourierMode::Vehicle,
    CourierMode::Bicycle,
    CourierMode::Walking,
];
pub(super) struct Relaxation {
    locations: Vec<LocationId>,
    distances: Vec<Vec<Vec<u128>>>,
}
impl Relaxation {
    pub fn new(
        p: &RoutingProblem,
        travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
        b: &mut Budget,
    ) -> Result<Self, ()> {
        let mut locations: Vec<_> = p
            .couriers
            .iter()
            .map(|c| c.depot)
            .chain(
                p.shipments
                    .iter()
                    .flat_map(|s| [s.pickup.location, s.delivery.location]),
            )
            .collect();
        locations.sort_by_key(|l| l.0);
        locations.dedup();
        let n = locations.len();
        let mut distances = Vec::new();
        for mode in MODES {
            let mut d = vec![vec![INF; n]; n];
            for (i, row) in d.iter_mut().enumerate() {
                for (j, cell) in row.iter_mut().enumerate() {
                    b.step()?;
                    *cell = if i == j {
                        0
                    } else {
                        travel(mode, locations[i], locations[j]).map_or(INF, u128::from)
                    };
                }
            }
            for k in 0..n {
                for i in 0..n {
                    for j in 0..n {
                        b.step()?;
                        d[i][j] = d[i][j].min(d[i][k] + d[k][j]);
                    }
                }
            }
            distances.push(d);
        }
        Ok(Self {
            locations,
            distances,
        })
    }
    fn duration(&self, mode: CourierMode, from: LocationId, to: LocationId) -> u128 {
        let m = MODES.iter().position(|&x| x == mode).expect("known mode");
        let i = self
            .locations
            .binary_search_by_key(&from.0, |l| l.0)
            .expect("problem location");
        let j = self
            .locations
            .binary_search_by_key(&to.0, |l| l.0)
            .expect("problem location");
        self.distances[m][i][j]
    }
    pub fn feasible(
        &self,
        p: &RoutingProblem,
        courier: usize,
        visits: &[(usize, bool)],
        b: &mut Budget,
    ) -> Result<bool, ()> {
        let c = &p.couriers[courier];
        let mut location = c.depot;
        let mut time = u128::from(c.shift.start);
        let mut load = 0u128;
        for &(job, pickup) in visits {
            b.step()?;
            let s = &p.shipments[job];
            if !s.allowed_modes.contains(&c.mode) {
                return Ok(false);
            }
            let stop = if pickup { &s.pickup } else { &s.delivery };
            let duration = self.duration(c.mode, location, stop.location);
            if duration == INF {
                return Ok(false);
            }
            time = (time + duration).max(u128::from(stop.window.start))
                + u128::from(stop.service_time);
            if time > u128::from(stop.window.end.min(c.shift.end)) {
                return Ok(false);
            }
            if pickup {
                load += u128::from(s.weight);
            } else {
                load -= u128::from(s.weight);
            }
            if load > u128::from(c.capacity) {
                return Ok(false);
            }
            location = stop.location;
        }
        b.step()?;
        Ok(time + self.duration(c.mode, location, c.depot) <= u128::from(c.shift.end))
    }
    pub fn conflict(
        &self,
        p: &RoutingProblem,
        c: usize,
        a: usize,
        z: usize,
        b: &mut Budget,
    ) -> Result<bool, ()> {
        let ap = (a, true);
        let ad = (a, false);
        let zp = (z, true);
        let zd = (z, false);
        for visits in [
            [ap, ad, zp, zd],
            [ap, zp, ad, zd],
            [ap, zp, zd, ad],
            [zp, zd, ap, ad],
            [zp, ap, zd, ad],
            [zp, ap, ad, zd],
        ] {
            if self.feasible(p, c, &visits, b)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
