using System.Text;
using System.Text.Json;

namespace Orchestrata;

internal static class ResultDecoder
{
    private const int MaxResponseBytes = (2 * 1024 * 1024) - 1; // Excludes the native C terminator.

    public static SolveOutcome Decode(string json, long[][] domains, ulong maxEvaluations)
    {
        if (Encoding.UTF8.GetByteCount(json) > MaxResponseBytes)
            throw new SolveResponseException();

        try
        {
            using JsonDocument document = JsonDocument.Parse(json, new JsonDocumentOptions { MaxDepth = 8 });
            JsonElement root = document.RootElement;
            RequireFields(root, "solution");
            JsonElement solution = root.GetProperty("solution");
            if (solution.ValueKind == JsonValueKind.Null)
                return new Infeasible();

            RequireFields(solution, "values", "score", "evaluated");
            JsonElement valuesJson = solution.GetProperty("values");
            if (valuesJson.ValueKind != JsonValueKind.Array || valuesJson.GetArrayLength() != domains.Length)
                throw new SolveResponseException();

            var values = new long[domains.Length];
            for (int index = 0; index < values.Length; index++)
            {
                long value = ReadSigned(valuesJson[index]);
                if (Array.IndexOf(domains[index], value) < 0)
                    throw new SolveResponseException();
                values[index] = value;
            }

            JsonElement score = solution.GetProperty("score");
            RequireFields(score, "hard", "soft");
            long hard = ReadSigned(score.GetProperty("hard"));
            long soft = ReadSigned(score.GetProperty("soft"));
            JsonElement evaluated = solution.GetProperty("evaluated");
            if (hard != 0 || soft > 0 || evaluated.ValueKind != JsonValueKind.Number
                || !evaluated.TryGetUInt64(out ulong count) || count == 0 || count > maxEvaluations)
                throw new SolveResponseException();

            return new Solved(new Solution(Array.AsReadOnly(values), new Score(hard, soft), count));
        }
        catch (JsonException)
        {
            throw new SolveResponseException();
        }
    }

    private static long ReadSigned(JsonElement element) =>
        element.ValueKind == JsonValueKind.Number && element.TryGetInt64(out long value)
            ? value
            : throw new SolveResponseException();

    private static void RequireFields(JsonElement value, params string[] expected)
    {
        if (value.ValueKind != JsonValueKind.Object)
            throw new SolveResponseException();

        var seen = new HashSet<string>(StringComparer.Ordinal);
        foreach (JsonProperty property in value.EnumerateObject())
        {
            if (!expected.Contains(property.Name, StringComparer.Ordinal) || !seen.Add(property.Name))
                throw new SolveResponseException();
        }
        if (seen.Count != expected.Length)
            throw new SolveResponseException();
    }
}
