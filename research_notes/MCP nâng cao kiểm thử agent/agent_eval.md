# Academic and Benchmark Evidence for Agent Evaluation in mcp-regress

## Which evaluation methods and metrics can strengthen deterministic tool and agent regression tests?

### Takeaway

The strongest pattern is layered scoring: validate call structure and schemas, verify observable state/effects, then measure reliability across repeated agent runs. For mcp-regress, deterministic protocol and server-contract assertions should remain the CI gate; model-driven task evaluation is a separate, optional diagnostic layer whose stochastic and environment-dependent scores must not be conflated with server compatibility.

### Cited Findings

- **BFCL (ICML 2025; conference held July 13–19, 2025)** evaluates serial and parallel function calls across programming languages with AST-based matching, curated and contributed functions, abstention, and stateful multi-step scenarios. Its main actionable idea for mcp-regress is to test parsed call structure and argument shape as explicit dimensions instead of reducing all failures to “tool mismatch.” [PMLR paper](https://proceedings.mlr.press/v267/patil25a.html)
- **τ-bench (arXiv preprint submitted June 17, 2024)** pairs API-equipped agents with simulated users and written domain policies, checks the final database state against an annotated goal state, and introduces pass^k: the probability that all k independent attempts succeed, averaged across tasks. The paper reports that GPT-4o succeeds on fewer than half of tasks and has pass^8 below 25% in retail, illustrating that one successful run does not establish reliability. [arXiv paper](https://arxiv.org/abs/2406.12045)
- τ-bench also explicitly identifies limits in its simulated user: language-model capacity, instruction ambiguity, and missing domain knowledge can affect the interaction and score. That makes a deterministic scripted user more appropriate for a CI gate, while LM-simulated users can be reserved for non-gating exploration. [τ-bench paper, discussion](https://arxiv.org/abs/2406.12045)
- **ToolSandbox (Findings of NAACL 2025; April 2025)** combines stateful tool execution, implicit dependencies between tools and world state, an on-policy conversational user simulator, and milestone evaluation over intermediate and final trajectory states. Its hard categories include state dependency, canonicalization, and insufficient information. This supports testing state transitions and intermediate invariants, not only comparing the final tool list. [ACL Anthology paper](https://aclanthology.org/2025.findings-naacl.65/)
- **CAREFUL-Bench (arXiv preprint v2 dated November 3, 2026)** infers function read/write behavior, idempotence, and state footprints by executing functions on sampled simulator states and comparing before/after snapshots; it reserves the judgment of whether a write is destructive for annotation. Its deterministic replay scorer counts unrequested destructive calls, duplicate non-idempotent writes, redundant reads, failed calls, and surplus calls, alongside the underlying task score. [CAREFUL-Bench paper](https://arxiv.org/html/2611.02978)
- CAREFUL-Bench reports that its effect-aware score preserves BFCL’s model ranking closely (Spearman ρ=0.98) while changing score gaps: the top model loses 14.5 points for unrequested destructive calls, and surplus steps consume 2–28% of input tokens. This shows that a useful diagnostic can preserve a base outcome metric while exposing behavior hidden by final-state-only scoring. The work is a preprint as of its v2 dated Nov. 3, 2026, not a peer-reviewed benchmark paper yet. [CAREFUL-Bench paper](https://arxiv.org/html/2611.02978)

### Inferences

- Add an effect-aware test profile as a separate report from the existing contract comparison: annotate fixture tools as read-only, write, idempotent/non-idempotent, destructive, and affected state fields; compare pre/post state and count duplicate writes, unnecessary reads, unexpected effects, and failed calls. This is a research-backed extension idea, not a result demonstrated on mcp-regress.
- Keep tool schema and protocol assertions deterministic and use them as the merge-blocking signal. A model-agent profile can report pass^k or pass^3, call efficiency, and failure categories as a separate score because repeated stochastic attempts and user simulation measure agent reliability rather than MCP server contract compatibility.
- Treat effect inference as evidence, not proof: probing sampled states can miss edge cases, while destructive classification is partly normative. Store the evidence and human decision behind each effect label, and use conservative labels when the test environment can cause irreversible actions.

### Gaps

- The cited studies evaluate agents against benchmark environments; they do not establish which effect-label schema or number of repeated attempts is optimal for a deterministic MCP server gate.
- These papers do not provide a direct controlled comparison of agent-evaluation CI gates versus conventional server contract tests, nor an agreed threshold for blocking a release on pass^k or effect violations.

## What do MCP-specific benchmarks reveal about realistic coverage, replay, and reproducibility?

### Takeaway

MCP evaluations make a strong case for realistic tool discovery, cross-server workflows, intermediate state/effect checkpoints, and failure taxonomies. They also show that “live and realistic” conflicts with reproducibility: pinning or replaying server behavior is necessary for stable comparisons, while live runs belong in a separately reported drift or interoperability lane.

### Cited Findings

- **MCP-Atlas (arXiv:2602.00933 v3 dated May 19, 2026; preprint)** contains 1,000 human-authored, expert-verified tasks over 36 real MCP servers and 220 tools. It typically exposes 6–37 candidate tools, of which only 2–8 are relevant; 98.6% of tasks require two or more servers. Prompts do not name the required tool or server, so the benchmark measures tool discovery and orchestration as well as execution. [MCP-Atlas paper](https://arxiv.org/html/2602.00933)
- MCP-Atlas scores atomic answer claims grounded in tool outputs, gives partial credit at claim level, and uses an 11-category diagnostic taxonomy spanning tool selection, malformed parameters, no tool use, failed error recovery, misunderstanding, synthesis, parsing, and early stopping. It publishes a 500-task public split and keeps 500 tasks private to monitor exposure. [MCP-Atlas paper](https://arxiv.org/html/2602.00933)
- MCP-Atlas runs real, version-pinned server containers with sandboxed file systems and allow-listed network egress, then restarts them between runs. Its authors report that all-LLM scoring varies by 2–5 percentage points across judges; live API/schema drift can make a task unsolvable or trivial; and its prompts and server responses are English-only. [MCP-Atlas paper, design and limitations](https://arxiv.org/html/2602.00933)
- **DynamicMCPBench (arXiv:2607.20531 v2 dated August 31, 2026; preprint)** generates goals, solves them on live MCP servers, and distills successful traces into effect checkpoints. Its deterministic first-tier replay scorer accepts equivalence sets and partial-order constraints rather than requiring one exact call path; each task is attempted three times and pass^3 requires all three to succeed. The benchmark study covered 24 models, 121 servers, and 750 tasks; authors report a chance-corrected human scoring agreement of 0.76. [DynamicMCPBench paper](https://arxiv.org/html/2607.20531)
- DynamicMCPBench’s replay boundary is explicit: a novel call without a recorded response cannot be served by its deterministic cache and is scored as failure, so the reported system is path-agnostic only among paths represented by its replay substrate. The authors also report that only 33% of recorded effects reproduced identically when 246 trajectories were re-executed against live services, demonstrating substantial live-data drift. [DynamicMCPBench paper](https://arxiv.org/html/2607.20531)
- DynamicMCPBench notes further limits: its human validity study covers one locally served model; state-changing tasks require sandboxing; some referenced state is absent at evaluation time; alternative-tool equivalence recall is unmeasured; and its public-API, mostly English substrate does not establish results for private or non-English deployments. [DynamicMCPBench paper](https://arxiv.org/html/2607.20531)
- **MCP-Bench (ICLR 2026; first arXiv version 2025)** connects agents to 28 MCP servers exposing 250 tools, creates multi-step tasks by deriving tool dependency chains from I/O signatures, then removes explicit tool directions to create fuzzy instructions. Its scoring combines deterministic checks for tool validity, schema compliance, runtime success, and dependency order with rubric-based LLM judgments of completion and planning. [ICLR paper](https://openreview.net/pdf?id=fe8mzHwMxN); [paper HTML](https://arxiv.org/html/2508.20453)
- **Live API-Bench (EACL 2026; March 2026)** turns BIRD-SQL queries into interactive API sequences across 11 databases and more than 2,500 invocable tools. It includes human-authored queries, gold API sequences, and verified final answers, targeting error handling, sequential reasoning, parameter generation, response parsing, and cross-domain robustness; reported completion rates were 7–47% for evaluated LLMs and up to 50% in interactive agent settings. [ACL Anthology paper](https://aclanthology.org/2026.eacl-long.143/)

### Inferences

- A practical product architecture is two-lane: (1) a small, deterministic, pinned server-contract suite that blocks regressions, and (2) an opt-in agent capability suite with realistic multi-tool tasks, repeated runs, richer diagnostics, and a separately versioned report. This prevents a changing external API or model version from making a server-contract CI result flaky.
- Curate test cases around failure modes rather than only tool count: unknown-tool distractors, malformed or omitted arguments, sequential dependencies, error recovery, incorrect server attribution, early stop, and synthesis from tool output. For reusable tests, retain task intent and allowed effects/equivalent paths in addition to an example trajectory.
- Keep a deterministic replay mode and a scheduled live drift mode distinct. Pin the server artifact, protocol/library versions, fixtures, and test data in replay mode; record timestamps and upstream versions in live mode. Treat an unreplayable alternative path as an evaluator coverage gap unless the task explicitly requires that path.
- Use human-authored or reviewed seed cases and held-out cases for score integrity; generated cases can expand coverage, but should be validated for solvability and grounded outputs before becoming trusted baselines.

### Gaps

- The cited MCP benchmark papers do not prove that their task collections cover the complete MCP protocol surface or replace a server conformance suite.
- Live API-Bench primarily exercises API sequences derived from SQL tasks, not the full MCP transport, lifecycle, capability negotiation, or server security boundary; mapping it to those concerns is an inference from the benchmark scope.
- The most recent MCP-focused work in this set (MCP-Atlas and DynamicMCPBench) is arXiv preprint research. Their methods are useful evidence, but their reported outcomes should be treated as preliminary until independently replicated.

## Which design choices should be adopted cautiously, and how should results be interpreted?

### Takeaway

Agent benchmarks measure the combined agent, prompt/scaffold, server, dataset, and evaluator. Their scores are useful for capability trends and failure discovery but are not direct evidence that an MCP server conforms to its contract. For a production-facing regression tool, prioritize deterministic oracles, expose evaluator uncertainty, and label which layer each result covers.

### Cited Findings

- MCP-Bench uses automated task synthesis and an LLM judge in its task-completion/planning evaluator, while MCP-Atlas identifies style bias and reduced reproducibility as risks of holistic LLM-as-judge scoring. MCP-Atlas instead scores atomic claims with multiple independent judges and still observes a 2–5 point cross-judge spread. [MCP-Bench paper](https://arxiv.org/html/2508.20453); [MCP-Atlas paper](https://arxiv.org/html/2602.00933)
- τ-bench’s user simulator is itself a language model, and the paper says ambiguity, missing user knowledge, and simulator reasoning or instruction-following limitations can affect results. [τ-bench paper](https://arxiv.org/abs/2406.12045)
- CAREFUL-Bench’s state-diff probes infer idempotence from sampled states, which its methods describe as evidence rather than proof; destructive classification requires judgment. [CAREFUL-Bench paper](https://arxiv.org/html/2611.02978)
- DynamicMCPBench’s own audit says its effect-equivalence sets have measured precision but not recall, so an unlisted equivalent path can fail deterministic scoring. It also reports reference inconsistencies in 104 of 750 tasks; removing them raises all pass^3 scores by about six points without changing ranking, affecting absolute difficulty. [DynamicMCPBench paper](https://arxiv.org/html/2607.20531)
- MCP-Atlas warns that its live-server leaderboard is a May 2026 snapshot: rate limits, auth, and schemas can change without notice. Even with version pins, task-level reproducibility is bounded by upstream stability. [MCP-Atlas paper](https://arxiv.org/html/2602.00933)

### Inferences

- Report at least three distinct result classes: deterministic protocol/contract pass-fail; deterministic behavioral/effect checks on a pinned fixture; and model-agent task scores with explicit model, prompt, adapter, tool exposure, server image, judge, seed/trial count, and run date. Do not merge these into one “compatibility” score.
- Do not make an LLM judge the authority for schema validity, protocol correctness, security-sensitive effects, or CI exit status. If an LLM judge is useful for claim-level task diagnostics, pin its model/version and publish disagreement or uncertainty alongside the score.
- Use every agent failure to propose a reproducible fixture, not automatically as a product defect: first establish whether the failure came from the server contract, client adapter, model, task ambiguity, upstream drift, or evaluator coverage.

### Gaps

- None of the cited papers specifies an MCP-specific, versioned standard for reporting protocol and server interoperability runs. The recommendation to separate these result classes is an engineering inference from the benchmarks’ differing targets and known sources of variability.
- The sources do not define a universal security policy for effects such as “destructive” or “requires confirmation”; those labels must be grounded in the product’s threat model and documented for each test environment.
