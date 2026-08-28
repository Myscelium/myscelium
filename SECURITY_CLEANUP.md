# History cleanup

Benchmark databases, historical SQLite files, logs, and machine/test metadata were previously committed and must be treated as exposed. Raw measurements are not republished.

The cleanup removes data-bearing paths from published branch and tag history:

- `*.db` and SQLite journal files
- `*.sqlite*`
- `*.log` and rotated log files

`NodeName`, `TestNodeName`, and `NodeDisk` remain in source code as application schema/API field names. Their historical database contents are removed. Renaming those fields would be a separate compatibility change.

Benchmark output now goes outside the repository. Set `MYSCELIUM_BENCHMARK_DB` to an absolute path to override the default `~/.cache/myscelium/benchmarks/test_results.db`.

Old clones remain contaminated. Contributors must reclone after rewritten refs are published and must not push from old clones.
