'use strict';

const { I64_MIN, I64_MAX, U64_MAX, inRange } = require('./integers');
const { SolveError } = require('./solve-error');

function invalid(path, expected) {
    throw new SolveError('invalid_input', `${path} must be ${expected}.`);
}

function record(value, fields, path) {
    if (value === null || typeof value !== 'object' || Array.isArray(value)
        || Object.keys(value).length !== fields.length
        || !Object.keys(value).every(field => fields.includes(field))) {
        invalid(path, `an object with exactly these fields: ${fields.join(', ')}`);
    }
}

function list(value, path, read) {
    if (!Array.isArray(value)) {
        invalid(path, 'an array');
    }
    return Array.from({ length: value.length }, (_, index) => read(value[index], `${path}[${index}]`));
}

function bigint(value, path, low = I64_MIN, high = I64_MAX) {
    if (!inRange(value, low, high)) {
        invalid(path, `a bigint between ${low} and ${high}`);
    }
    return value;
}

function number(value, path, high) {
    if (!Number.isSafeInteger(value) || value < 0 || value > high) {
        invalid(path, `an integer Number from 0 through ${high}`);
    }
    return value;
}

function raw(value) {
    return JSON.rawJSON(value.toString());
}

function signed(value, path) {
    return raw(bigint(value, path));
}

function unsigned(value, path) {
    return raw(bigint(value, path, 0n, U64_MAX));
}

function weight(value, path) {
    return raw(bigint(value, path, 1n, I64_MAX));
}

function encodeConstraint(constraint, path, variableCount) {
    const index = (value, location) => number(value, location, variableCount - 1);
    const indices = value => list(value, `${path}.variables`, index);
    const field = (...fields) => record(constraint, ['kind', ...fields], path);
    const kind = constraint?.kind;
    switch (kind) {
        case 'all_different':
            field('variables', 'hardWeight');
            return { kind, variables: indices(constraint.variables), hard_weight: weight(constraint.hardWeight, `${path}.hardWeight`) };
        case 'not_equal_offset':
            field('left', 'right', 'offset', 'hardWeight');
            return {
                kind, left: index(constraint.left, `${path}.left`), right: index(constraint.right, `${path}.right`),
                offset: signed(constraint.offset, `${path}.offset`), hard_weight: weight(constraint.hardWeight, `${path}.hardWeight`),
            };
        case 'prefer_value':
            field('variable', 'value', 'softWeight');
            return {
                kind, variable: index(constraint.variable, `${path}.variable`),
                value: signed(constraint.value, `${path}.value`), soft_weight: weight(constraint.softWeight, `${path}.softWeight`),
            };
        case 'count_value': {
            field('variables', 'value', 'min', 'max', 'weight', 'tier');
            if (constraint.tier !== 'hard' && constraint.tier !== 'soft') {
                invalid(`${path}.tier`, '"hard" or "soft"');
            }
            const variables = indices(constraint.variables);
            return {
                kind, variables, value: signed(constraint.value, `${path}.value`),
                min: number(constraint.min, `${path}.min`, variables.length),
                max: number(constraint.max, `${path}.max`, variables.length),
                weight: weight(constraint.weight, `${path}.weight`), tier: constraint.tier,
            };
        }
        case 'precedence':
            field('before', 'after', 'minGap');
            return {
                kind, before: index(constraint.before, `${path}.before`), after: index(constraint.after, `${path}.after`),
                min_gap: unsigned(constraint.minGap, `${path}.minGap`),
            };
        case 'no_overlap':
            field('intervals');
            return {
                kind,
                intervals: list(constraint.intervals, `${path}.intervals`, (interval, location) => {
                    record(interval, ['start', 'duration'], location);
                    return { start: index(interval.start, `${location}.start`), duration: unsigned(interval.duration, `${location}.duration`) };
                }),
            };
        case 'resource_capacity':
            field('items', 'limits');
            return {
                kind,
                items: list(constraint.items, `${path}.items`, (item, location) => {
                    record(item, ['variable', 'load'], location);
                    return { variable: index(item.variable, `${location}.variable`), load: unsigned(item.load, `${location}.load`) };
                }),
                limits: list(constraint.limits, `${path}.limits`, (limit, location) => {
                    record(limit, ['resource', 'capacity'], location);
                    return { resource: signed(limit.resource, `${location}.resource`), capacity: unsigned(limit.capacity, `${location}.capacity`) };
                }),
            };
        default:
            return invalid(`${path}.kind`, 'a supported constraint kind');
    }
}

function encodeProblem(problem) {
    record(problem, ['variables', 'constraints'], 'problem');
    const variables = list(problem.variables, 'problem.variables', (variable, path) => {
        record(variable, ['name', 'domain'], path);
        if (typeof variable.name !== 'string' || !variable.name.trim()) {
            invalid(`${path}.name`, 'a nonblank string');
        }
        const domain = list(variable.domain, `${path}.domain`, bigint);
        if (domain.length === 0) {
            invalid(`${path}.domain`, 'a nonempty array');
        }
        return { name: variable.name, domain };
    });
    const constraints = list(problem.constraints, 'problem.constraints', (value, path) =>
        encodeConstraint(value, path, variables.length));
    return {
        json: JSON.stringify({
            variables: variables.map(variable => ({ name: variable.name, domain: variable.domain.map(raw) })),
            constraints,
        }),
        domains: variables.map(variable => variable.domain),
    };
}

function requireBudget(budget) {
    return bigint(budget, 'maxEvaluations', 0n, U64_MAX);
}

module.exports = { encodeProblem, requireBudget };
