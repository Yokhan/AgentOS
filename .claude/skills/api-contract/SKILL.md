---
name: api-contract
description: "Validate API implementations against contracts in docs/API_CONTRACTS.md. Trigger when API endpoints change or user asks to verify API."
---

# API Contract Validation Skill

Ensure API implementations match documented contracts.

## Process

### Phase 1: Load Contracts
1. Read `docs/API_CONTRACTS.md`
2. Parse each endpoint: method, path, request schema, response schema, status codes

### Phase 2: Find Implementations
1. Search codebase for route definitions matching documented paths
2. Map each contract to its implementation file

### Phase 3: Validate
For each endpoint:
- **Request schema**: Does implementation validate all required fields?
- **Response schema**: Does implementation return all documented fields?
- **Status codes**: Are all documented error codes handled?
- **Types**: Do TypeScript/Python types match the contract?

### Phase 4: Report Mismatches
For each mismatch:
1. Show contract vs implementation
2. Ask user: update CONTRACT or update CODE?
3. Apply the chosen fix

### Phase 5: Generate Missing Contracts
If implementation exists but no contract:
1. Analyze the endpoint implementation
2. Generate contract entry for docs/API_CONTRACTS.md
3. Include: method, path, request/response schemas, status codes

### Phase 6: Generate OpenAPI Spec
If `docs/openapi.yaml` does not exist:
1. Use contracts from `docs/API_CONTRACTS.md` as source of truth
2. Generate `docs/openapi.yaml` with: paths, methods, request/response schemas, status codes, auth requirements
3. Validate generated spec against implementation

## Sync with Tests
After contract changes:
1. Check if integration tests exist for the endpoint
2. Verify tests match updated contract
3. Flag untested endpoints

---

## API Versioning Strategies

### URL Path Versioning (`/api/v1/users`)
- **Pros**: Explicit, easy to route, easy to cache, easy to deprecate
- **Cons**: URL pollution, clients must update URLs, harder to share code between versions
- **Best for**: Public APIs with many external consumers, APIs where versions diverge significantly

### Header Versioning (`Accept: application/vnd.myapi.v2+json`)
- **Pros**: Clean URLs, version negotiation possible, more RESTful
- **Cons**: Harder to test (cannot just paste a URL), easy to forget the header, less visible
- **Best for**: Internal APIs, APIs where versions are mostly compatible

### Query Parameter (`/api/users?version=2`)
- **Pros**: Easy to add, works everywhere, easy to default
- **Cons**: Looks like a filter not a version, caching complications, messy
- **Best for**: Quick versioning for simple APIs, transitional use only

### Content Negotiation (`Accept: application/json; version=2`)
- **Pros**: Standard HTTP mechanism, flexible
- **Cons**: Complex implementation, poor tooling support, hard to discover
- **Best for**: Sophisticated APIs with content-type variation needs

**Recommendation**: URL path versioning for most projects. It is the most explicit and debuggable. Only move to header versioning when you have a strong reason (many versions, minimal divergence).

## Breaking vs Non-Breaking Changes Checklist

### Non-Breaking (Safe to Deploy Without Version Bump)
- Adding a new optional field to a response
- Adding a new endpoint
- Adding a new optional query parameter
- Adding a new enum value to a response field (if consumers handle unknown values)
- Widening a numeric type (int32 to int64)
- Increasing a string length limit
- Adding a new HTTP method to an existing resource

### Breaking (Requires Version Bump or Migration Period)
- Removing or renaming a response field
- Removing or renaming an endpoint
- Adding a new REQUIRED field to a request
- Changing a field type (string to number)
- Changing the meaning of an existing field
- Removing an enum value from a response field
- Changing authentication mechanism
- Changing error response format
- Narrowing a numeric type (int64 to int32)
- Reducing a rate limit

### Gray Area (Depends on Consumer Contracts)
- Adding a new enum value to a REQUEST field (breaks if consumer validates strictly)
- Changing field ordering in responses (breaks if consumer parses positionally)
- Adding required headers (breaks existing clients)
- Changing default values (breaks consumers relying on defaults)

## OpenAPI/Swagger Generation Workflow

### From Existing Code (Code-First)
1. Annotate route handlers with schema decorators/comments
2. Generate OpenAPI spec: `npx swagger-jsdoc` / `python manage.py generateschema` / framework-specific tool
3. Validate generated spec: `npx @redocly/cli lint openapi.yaml`
4. Compare against `docs/API_CONTRACTS.md` for mismatches
5. Serve documentation: Swagger UI at `/api/docs` in development

### From Contract (Contract-First)
1. Write OpenAPI spec in `docs/openapi.yaml` based on `docs/API_CONTRACTS.md`
2. Generate types/interfaces: `npx openapi-typescript openapi.yaml -o types.ts` or `openapi-generator`
3. Implement handlers that conform to generated types
4. Validate at CI time: compare implementation against spec
5. Auto-generate client SDKs from spec if needed

### Keeping Spec in Sync
- CI check: `openapi-diff old-spec.yaml new-spec.yaml` to detect unintended changes
- Pre-commit hook: regenerate and compare (fail if drift detected)
- Contract tests: programmatically validate responses match spec schemas at test time

## Consumer-Driven Contract Testing

Traditional testing validates "does my API work?" Consumer-driven contracts validate "does my API work FOR MY CONSUMERS?"

### Concept
1. Each consumer defines a contract: "I call GET /users/123 and expect {id, name, email} in the response"
2. The provider runs ALL consumer contracts in its CI pipeline
3. If a provider change breaks any consumer contract, CI fails BEFORE deployment
4. Consumers can evolve their contracts independently

### Implementation Options
- **Pact**: most mature, supports many languages, uses a broker for contract exchange
- **Spring Cloud Contract**: JVM-focused, generates tests from Groovy/YAML contracts
- **Schema validation**: lightweight -- validate responses against JSON Schema in integration tests

### When to Use
- Multiple teams consume your API
- API breaking changes have caused production incidents before
- You want to decouple provider and consumer deployment schedules
- You need confidence that deprecating a field will not break consumers

## Error Response Standardization (RFC 7807)

Use RFC 7807 Problem Details for ALL error responses. This gives consumers a consistent, parseable error format.

### Format
```json
{
  "type": "https://api.example.com/problems/validation-error",
  "title": "Validation Error",
  "status": 422,
  "detail": "Field email must be a valid email address.",
  "instance": "/api/users/signup",
  "errors": [
    {
      "field": "email",
      "message": "Must be a valid email address",
      "code": "INVALID_FORMAT"
    }
  ]
}
```

### Fields
- **type** (URI): machine-readable error category, links to documentation
- **title** (string): short human-readable summary (same for all instances of this type)
- **status** (integer): HTTP status code (duplicated for convenience)
- **detail** (string): human-readable explanation specific to THIS occurrence
- **instance** (string): URI identifying this specific occurrence (for logging/support)
- **extensions**: additional fields like errors[] for validation, retryAfter for rate limits

### Standard Error Types to Define
- `validation-error` (422): request body/params failed validation
- `not-found` (404): requested resource does not exist
- `unauthorized` (401): missing or invalid authentication
- `forbidden` (403): authenticated but not authorized
- `conflict` (409): state conflict (duplicate, concurrent modification)
- `rate-limited` (429): too many requests, include retryAfter
- `internal-error` (500): unhandled server error (never leak stack traces)

## Rate Limiting and Pagination Patterns

### Rate Limiting
Communicate limits via standard headers:
```
X-RateLimit-Limit: 100        # requests per window
X-RateLimit-Remaining: 42     # requests left in current window
X-RateLimit-Reset: 1620000000 # Unix timestamp when window resets
Retry-After: 30               # seconds to wait (on 429 response)
```

Strategies:
- **Fixed window**: simple, but allows bursts at window boundaries
- **Sliding window**: smoother, prevents boundary bursts
- **Token bucket**: allows controlled bursts, most flexible
- **Per-user vs per-IP**: authenticate first, then rate limit by user identity

### Pagination
**Cursor-based** (recommended for most cases):
```json
{
  "data": [],
  "pagination": {
    "next_cursor": "eyJpZCI6MTAwfQ==",
    "has_more": true
  }
}
```
- Stable under inserts/deletes, performant (no OFFSET), good for real-time feeds
- Downside: no random page access, no total count without extra query

**Offset-based** (use when random access needed):
```json
{
  "data": [],
  "pagination": {
    "total": 1234,
    "page": 3,
    "per_page": 20
  }
}
```
- Familiar, supports "jump to page 5", easy total count
- Downside: slow at large offsets (OFFSET 10000), unstable under concurrent writes

**Keyset pagination** (high-performance variant of cursor):
- Use indexed column as cursor: `WHERE id > last_seen_id ORDER BY id LIMIT 20`
- Best performance, stable, but requires a unique, sortable column

## Cross-Reference

- `.claude/library/meta/strategic-thinking.md` -- "Positive-sum framing": API changes should serve both provider and consumer. Never break consumers for provider convenience. "Iterated games": API relationships are long-term; trust compounds.
- `.claude/library/domain/domain-guards.md` -- "Fail Fast": validate API inputs at boundaries. "Vertical Slice Architecture": each feature owns its endpoints.
- `.claude/library/technical/error-handling.md` -- Typed errors per domain, logging at boundaries, recovery strategies
