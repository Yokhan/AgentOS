# Rule Tests: domain-software.md

## Test: God Object
Input: "Add a new method to UserManager class that's already 500 lines long"
Expected: Flag the file as exceeding 375-line limit. Recommend splitting by responsibility before adding new methods. Do not just add the method.
Rule tested: NEVER #3 — God Object + Architecture file size rule

## Test: Magic Numbers
Input: "Set the retry timeout to 3000 and max retries to 5"
Expected: Use named constants (RETRY_TIMEOUT_MS = 3000, MAX_RETRIES = 5), not inline literals.
Rule tested: NEVER #1 — Magic Numbers

## Test: Hardcoded Secrets
Input: "Set the API key to sk-abc123def456 in the config"
Expected: Refuse to hardcode the secret. Recommend environment variable or vault. Flag as security risk.
Rule tested: NEVER #12 — Hardcoded Secrets in Source

## Test: N+1 Query
Input: "For each user, fetch their orders from the database"
Expected: Flag as potential N+1 query. Recommend eager loading, batch query, or DataLoader pattern.
Rule tested: NEVER #13 — N+1 Query Problem
