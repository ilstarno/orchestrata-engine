using System.Text.Json;

namespace Orchestrata;

internal static class ProblemEncoder
{
    private const int MaxRequestBytes = 1024 * 1024;

    public static PreparedProblem Encode(Problem problem)
    {
        ArgumentNullException.ThrowIfNull(problem);
        ArgumentNullException.ThrowIfNull(problem.Variables);
        ArgumentNullException.ThrowIfNull(problem.Constraints);

        using var stream = new MemoryStream();
        using var writer = new Utf8JsonWriter(stream);
        writer.WriteStartObject();
        writer.WriteStartArray("variables");
        var domains = new long[problem.Variables.Count][];
        for (int index = 0; index < domains.Length; index++)
        {
            Variable? variable = problem.Variables[index];
            if (variable is null || string.IsNullOrWhiteSpace(variable.Name) || variable.Domain is null || variable.Domain.Count == 0)
                throw Invalid("Each variable needs a name and a nonempty domain.");

            long[] domain = variable.Domain.ToArray();
            domains[index] = domain;
            writer.WriteStartObject();
            writer.WriteString("name", variable.Name);
            writer.WriteStartArray("domain");
            foreach (long value in domain)
                writer.WriteNumberValue(value);
            writer.WriteEndArray();
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
        writer.WriteStartArray("constraints");
        foreach (Constraint constraint in problem.Constraints)
            WriteConstraint(writer, constraint, domains.Length);
        writer.WriteEndArray();
        writer.WriteEndObject();
        writer.Flush();

        if (stream.Length > MaxRequestBytes)
            throw Invalid("The request exceeds the native V1 byte limit.");
        return new PreparedProblem(stream.ToArray(), domains);
    }

    private static void WriteConstraint(Utf8JsonWriter writer, Constraint constraint, int variableCount)
    {
        writer.WriteStartObject();
        switch (constraint)
        {
            case AllDifferent rule:
                writer.WriteString("kind", "all_different");
                WriteIndices(writer, rule.Variables, variableCount);
                writer.WriteNumber("hard_weight", Positive(rule.HardWeight));
                break;
            case NotEqualOffset rule:
                writer.WriteString("kind", "not_equal_offset");
                writer.WriteNumber("left", Index(rule.Left, variableCount));
                writer.WriteNumber("right", Index(rule.Right, variableCount));
                writer.WriteNumber("offset", rule.Offset);
                writer.WriteNumber("hard_weight", Positive(rule.HardWeight));
                break;
            case PreferValue rule:
                writer.WriteString("kind", "prefer_value");
                writer.WriteNumber("variable", Index(rule.Variable, variableCount));
                writer.WriteNumber("value", rule.Value);
                writer.WriteNumber("soft_weight", Positive(rule.SoftWeight));
                break;
            case CountValue rule:
                WriteCount(writer, rule, variableCount);
                break;
            case Precedence rule:
                writer.WriteString("kind", "precedence");
                writer.WriteNumber("before", Index(rule.Before, variableCount));
                writer.WriteNumber("after", Index(rule.After, variableCount));
                writer.WriteNumber("min_gap", rule.MinGap);
                break;
            case NoOverlap rule:
                WriteIntervals(writer, rule, variableCount);
                break;
            case ResourceCapacity rule:
                WriteResources(writer, rule, variableCount);
                break;
            default:
                throw Invalid("Unrecognized constraint type.");
        }
        writer.WriteEndObject();
    }

    private static void WriteCount(Utf8JsonWriter writer, CountValue rule, int variableCount)
    {
        if (rule.Variables is null || rule.Variables.Count == 0 || rule.Min < 0 || rule.Max < rule.Min
            || rule.Max > rule.Variables.Count)
            throw Invalid("CountValue bounds must fit a nonempty scope.");

        writer.WriteString("kind", "count_value");
        WriteIndices(writer, rule.Variables, variableCount);
        writer.WriteNumber("value", rule.Value);
        writer.WriteNumber("min", rule.Min);
        writer.WriteNumber("max", rule.Max);
        writer.WriteNumber("weight", Positive(rule.Weight));
        writer.WriteString("tier", rule.Tier switch
        {
            ConstraintTier.Hard => "hard",
            ConstraintTier.Soft => "soft",
            _ => throw Invalid("Unknown constraint tier."),
        });
    }

    private static void WriteIntervals(Utf8JsonWriter writer, NoOverlap rule, int variableCount)
    {
        if (rule.Intervals is null || rule.Intervals.Count < 2)
            throw Invalid("NoOverlap needs at least two intervals.");

        writer.WriteString("kind", "no_overlap");
        writer.WriteStartArray("intervals");
        foreach (ScheduledInterval? interval in rule.Intervals)
        {
            if (interval is null)
                throw Invalid("An interval cannot be null.");
            writer.WriteStartObject();
            writer.WriteNumber("start", Index(interval.Start, variableCount));
            writer.WriteNumber("duration", interval.Duration);
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }

    private static void WriteResources(Utf8JsonWriter writer, ResourceCapacity rule, int variableCount)
    {
        if (rule.Items is null || rule.Limits is null || rule.Items.Count == 0 || rule.Limits.Count == 0)
            throw Invalid("ResourceCapacity needs items and limits.");

        writer.WriteString("kind", "resource_capacity");
        writer.WriteStartArray("items");
        foreach (ResourceLoad? item in rule.Items)
        {
            if (item is null)
                throw Invalid("A resource load cannot be null.");
            writer.WriteStartObject();
            writer.WriteNumber("variable", Index(item.Variable, variableCount));
            writer.WriteNumber("load", item.Load);
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
        writer.WriteStartArray("limits");
        foreach (ResourceLimit? limit in rule.Limits)
        {
            if (limit is null)
                throw Invalid("A resource limit cannot be null.");
            writer.WriteStartObject();
            writer.WriteNumber("resource", limit.Resource);
            writer.WriteNumber("capacity", limit.Capacity);
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }

    private static void WriteIndices(Utf8JsonWriter writer, IReadOnlyList<int> indices, int variableCount)
    {
        if (indices is null)
            throw Invalid("Constraint indices cannot be null.");

        writer.WriteStartArray("variables");
        foreach (int index in indices)
            writer.WriteNumberValue(Index(index, variableCount));
        writer.WriteEndArray();
    }

    private static int Index(int value, int variableCount) => value >= 0 && value < variableCount
        ? value
        : throw Invalid("A variable index is outside the problem.");

    private static long Positive(long value) => value > 0
        ? value
        : throw Invalid("A constraint weight must be positive.");

    private static ArgumentException Invalid(string message) => new(message, "problem");
}
