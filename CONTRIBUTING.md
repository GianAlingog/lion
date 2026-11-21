# Contributing

The `main` branch should always have a passing CI, appended to by branches and PRs.

## Branches

```
<tag>/<goal>
```

`<goal>` should be a few words to describe the outcome of the merging branch.

| Tag | Use for |
|---|---|
| `feat` | Add a new feature |
| `fix` | Resolve incorrect behavior |
| `test` | Write tests |
| `bench` | Write benchmarks or add results |
| `perf` | Improve performance but don't change behavior |
| `refactor` | Change project structure but don't change behavior |
| `docs` | Write documentation (README, built-in, etc) |
| `misc` | Miscellaneous (CI, Tooling, etc) |

*Avoid making commits with only formatter or linter changes.

## Pull Requests

Title: `<tag>: <summary>`

Use the template provided for the description.

CI should be green before merging.