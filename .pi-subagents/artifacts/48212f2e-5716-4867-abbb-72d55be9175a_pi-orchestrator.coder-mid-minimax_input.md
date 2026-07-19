# Task for pi-orchestrator.coder-mid-minimax

Refactor src/web/mod.rs (4181 lines) to introduce a custom AppError type that implements From<anyhow::Error> and IntoResponse, eliminating the per-handler `.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::with_details("Failed to ...", e.to_string())))?` boilerplate.

Location: /Users/svenlochner/dev/LazyQMK/src/web/mod.rs

PROBLEM: ~40 HTTP handlers in this file follow this pattern:
```rust
async fn some_handler(...) -> Result<Json<T>, (StatusCode, Json<ApiError>)> {
    do_thing().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::with_details("Failed to do thing", e.to_string()))))?;
    ...
}
```

GOAL: Refactor to:
```rust
async fn some_handler(...) -> Result<Json<T>, AppError> {
    do_thing()?;  // Anyhow::Error auto-converts via From
    ...
}

#[derive(Debug)]
enum AppError {
    BadRequest(String),  // 400
    NotFound(String),    // 404
    Internal(anyhow::Error),  // 500 with debug details
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg, details) = match &self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone(), None),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone(), None),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), Some(e.to_string())),
        };
        (status, Json(ApiError::with_details(&msg, details.as_deref().unwrap_or("")))).into_response()
    }
}
```

Each handler then becomes:
```rust
async fn some_handler(...) -> Result<Json<T>, AppError> {
    let x = do_thing()?;  // uses From<anyhow::Error>
    Ok(Json(x))
}
```

For error cases that need specific status codes, use:
```rust
return Err(AppError::NotFound(format!("Layout not found: {filename}")));
return Err(AppError::BadRequest(format!("Invalid layer: {layer}")));
```

STEPS:
1. Read /Users/svenlochner/dev/LazyQMK/src/web/mod.rs entirely (~4181 lines)
2. Define AppError enum + impls near top of file (after existing ApiError definition)
3. Refactor each of the ~40 async fn handlers one-by-one. For each:
   - Change return type from `Result<_, (StatusCode, Json<ApiError>)>` to `Result<_, AppError>`
   - Replace `.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::with_details("Failed to X", e.to_string())))?` with `?`
   - Replace `.map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiError::new("..."))))?` with `?` followed by an error category check (or direct `AppError::BadRequest`)
   - Replace `(StatusCode::NOT_FOUND, Json(ApiError::new(format!("X not found"))))` returns with `AppError::NotFound(...)`
4. Run `cargo test --test web_api_tests 2>&1 | tail -10` — MUST show all 1928-line web_api_tests pass (this is the highest-stakes test; do not break it)
5. Run `cargo test 2>&1 | tail -3` — full suite
6. Run `cargo clippy --all-features -- -D warnings 2>&1 | tail -3` — must be clean
7. Verify with `rg 'StatusCode, Json' src/web/mod.rs | wc -l` — must drop dramatically (from 120 to single-digit)
8. Verify with `rg 'ApiError::with_details' src/web/mod.rs | wc -l` — must drop dramatically

CRITICAL CONSTRAINTS:
- Do NOT change the public test surface. Tests call HTTP endpoints and assert on response statuses + body content. The `AppError` machinery must produce equivalent responses (same status codes for same error categories).
- Tests likely inspect JSON body content like `ApiError` serialization. Preserve the JSON shape: `{message, details?}`.
- Do NOT edit tests/, src/web/build_jobs.rs, src/web/generate_jobs.rs, or any router setup. Only touch src/web/mod.rs.
- Do NOT add new dependencies. Use only what's available: anyhow, axum.
- Do NOT remove `pub struct ApiError` or its serde derives — it's part of the test surface.
- Preserve ALL existing handler behavior. A handler that returned 400 with "Invalid file name" must STILL return 400 with "Invalid file name" after refactor.

REFACTOR SCOPE: There are ~40 handlers in the file. Refactor them all mechanically. The handlers may use `axum::extract::Json`, `axum::extract::Path`, `axum::extract::Query`, `axum::extract::State`, etc. — handle each carefully.

When you're done, list any handler you couldn't refactor cleanly and the reason. Mark with `// TODO(refactor): <reason>` comments.

OUTPUT FINAL RESPONSE:
- `git diff --stat src/web/mod.rs | head -3`
- tests/web_api_tests result (must be unchanged count)
- cargo test result (1474 must be unchanged)
- cargo clippy result (must be clean)
- count of remaining `StatusCode, Json` and `ApiError::with_details` in the file
- list of any handlers you couldn't fully refactor

cwd: /Users/svenlochner/dev/LazyQMK. Use context "fresh".

---
**Output:**
Write your findings to exactly this path: /tmp/minimax-apperror.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: required by reviewer.

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```