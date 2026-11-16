use std::collections::{HashMap, HashSet};

use crate::{Error, Problem, ResourceCapacityError, ResourceLimit, ResourceLoad};

pub(super) fn validate_resource_capacity(
    problem: &Problem,
    items: &[ResourceLoad],
    limits: &[ResourceLimit],
) -> Result<(), Error> {
    if items.is_empty() {
        return Err(Error::InvalidResourceCapacity(
            ResourceCapacityError::EmptyItems,
        ));
    }

    let mut resources = HashSet::with_capacity(limits.len());
    for limit in limits {
        if !resources.insert(limit.resource) {
            return Err(Error::InvalidResourceCapacity(
                ResourceCapacityError::DuplicateLimit(limit.resource),
            ));
        }
    }

    let mut potential = HashMap::<i64, u64>::with_capacity(limits.len());
    let mut variables = HashSet::with_capacity(items.len());
    for item in items {
        let variable = problem
            .variables
            .get(item.variable)
            .ok_or(Error::InvalidVariable(item.variable))?;
        if !variables.insert(item.variable) {
            return Err(Error::InvalidResourceCapacity(
                ResourceCapacityError::DuplicateItem(item.variable),
            ));
        }
        let mut domain = HashSet::with_capacity(variable.domain.len());
        for &resource in &variable.domain {
            if !domain.insert(resource) {
                return Err(Error::InvalidResourceCapacity(
                    ResourceCapacityError::DuplicateDomainValue {
                        variable: item.variable,
                        resource,
                    },
                ));
            }
            if !resources.contains(&resource) {
                return Err(Error::InvalidResourceCapacity(
                    ResourceCapacityError::MissingLimit(resource),
                ));
            }
            let total = potential.entry(resource).or_default();
            *total = total
                .checked_add(item.load)
                .ok_or(Error::InvalidResourceCapacity(
                    ResourceCapacityError::PotentialLoadOverflow(resource),
                ))?;
        }
    }

    for limit in limits {
        if !potential.contains_key(&limit.resource) {
            return Err(Error::InvalidResourceCapacity(
                ResourceCapacityError::UnusedLimit(limit.resource),
            ));
        }
    }
    Ok(())
}
