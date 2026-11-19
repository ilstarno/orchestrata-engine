using System.Text.Json;
using Orchestrata;

using var solver = new Solver(args[0]);
const string problem = """{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}""";
using var response = JsonDocument.Parse(solver.SolveJson(problem, 1));
long value = response.RootElement.GetProperty("solution").GetProperty("values")[0].GetInt64();
if (value != long.MaxValue)
    throw new Exception("The solver lost integer precision.");

TypedTests.Expect<SolveException>(() => solver.SolveJson(problem, 0), error => error.Status == 2);
TypedTests.Run(solver);
if (args.Length == 2)
    TypedTests.RejectMalformedResponses(args[1]);
Console.WriteLine("C# raw and typed native tests passed.");
