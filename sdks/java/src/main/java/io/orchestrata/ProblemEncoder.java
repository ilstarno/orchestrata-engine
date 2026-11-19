package io.orchestrata;

import java.nio.charset.StandardCharsets;
import java.util.HashSet;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

final class ProblemEncoder {
    byte[] encode(Problem problem) {
        Objects.requireNonNull(problem, "problem");
        String variables = problem.variables().stream().map(v -> "{\"name\":" + quote(v.name())
            + ",\"domain\":" + v.domain() + "}").collect(Collectors.joining(","));
        int size = problem.variables().size();
        String constraints = problem.constraints().stream().map(c -> constraint(c, size))
            .collect(Collectors.joining(","));
        return ("{\"variables\":[" + variables + "],\"constraints\":[" + constraints + "]}")
            .getBytes(StandardCharsets.UTF_8);
    }

    private String constraint(Constraint constraint, int size) {
        return switch (constraint) {
            case AllDifferent c -> {
                c.variables().forEach(index -> Objects.checkIndex(index, size));
                yield "{\"kind\":\"all_different\",\"variables\":" + c.variables()
                    + ",\"hard_weight\":" + c.hardWeight() + "}";
            }
            case NotEqualOffset c -> "{\"kind\":\"not_equal_offset\",\"left\":"
                + Objects.checkIndex(c.left(), size) + ",\"right\":" + Objects.checkIndex(c.right(), size)
                + ",\"offset\":" + c.offset() + ",\"hard_weight\":" + c.hardWeight() + "}";
            case PreferValue c -> "{\"kind\":\"prefer_value\",\"variable\":"
                + Objects.checkIndex(c.variable(), size) + ",\"value\":" + c.value()
                + ",\"soft_weight\":" + c.softWeight() + "}";
            case CountValue c -> {
                checkDistinct(c.variables(), size);
                yield "{\"kind\":\"count_value\",\"variables\":" + c.variables()
                    + ",\"value\":" + c.value() + ",\"min\":" + c.min() + ",\"max\":" + c.max()
                    + ",\"weight\":" + c.weight() + ",\"tier\":\"" + c.tier().wireName + "\"}";
            }
            case Precedence c -> {
                checkDistinct(List.of(c.before(), c.after()), size);
                yield "{\"kind\":\"precedence\",\"before\":" + c.before()
                    + ",\"after\":" + c.after() + ",\"min_gap\":" + c.minGap() + "}";
            }
            case NoOverlap c -> {
                checkDistinct(c.intervals().stream().map(ScheduledInterval::start).toList(), size);
                String intervals = c.intervals().stream()
                    .map(i -> "{\"start\":" + i.start() + ",\"duration\":" + i.duration() + "}")
                    .collect(Collectors.joining(","));
                yield "{\"kind\":\"no_overlap\",\"intervals\":[" + intervals + "]}";
            }
            case ResourceCapacity c -> {
                checkDistinct(c.items().stream().map(ResourceLoad::variable).toList(), size);
                var resources = new HashSet<Long>();
                for (var limit : c.limits()) {
                    if (!resources.add(limit.resource())) {
                        throw new IllegalArgumentException("Duplicate resource limit");
                    }
                }
                String items = c.items().stream()
                    .map(i -> "{\"variable\":" + i.variable() + ",\"load\":" + i.load() + "}")
                    .collect(Collectors.joining(","));
                String limits = c.limits().stream()
                    .map(i -> "{\"resource\":" + i.resource() + ",\"capacity\":" + i.capacity() + "}")
                    .collect(Collectors.joining(","));
                yield "{\"kind\":\"resource_capacity\",\"items\":[" + items
                    + "],\"limits\":[" + limits + "]}";
            }
        };
    }

    private void checkDistinct(List<Integer> indices, int size) {
        var seen = new HashSet<Integer>();
        for (int index : indices) {
            Objects.checkIndex(index, size);
            if (!seen.add(index)) throw new IllegalArgumentException("Duplicate variable index");
        }
    }

    private String quote(String value) {
        if (!StandardCharsets.UTF_8.newEncoder().canEncode(value)) {
            throw new IllegalArgumentException("Expected a valid Unicode variable name");
        }
        StringBuilder escaped = new StringBuilder("\"");
        value.chars().forEach(c -> {
            if (c == '"' || c == '\\') escaped.append('\\').append((char) c);
            else if (c < 32) escaped.append(String.format("\\u%04x", c));
            else escaped.append((char) c);
        });
        return escaped.append('"').toString();
    }
}
