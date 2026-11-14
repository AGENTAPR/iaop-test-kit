# iaop-test-kit

Integration testing utilities and common test infrastructure.

## Purpose

iaop-test-kit is part of the **Insurance Agent Operating Platform (IAOP)**, an enterprise
insurance agent lifecycle platform.

## Bounded Context / Ownership

- **Bounded context:** Test Infrastructure
- **Owner:** AGENTAPR
- **Group:** shared-library

## Owned Database

N/A (no owned database)

Each service owns its persistence. No cross-service database access, no shared domain
entities, and no cross-database foreign keys.

## Technical Stack

Rust

## Architecture Principles

- Domain-Driven Design (DDD) with explicit bounded contexts
- Hexagonal architecture (ports and adapters)
- Event-driven integration via versioned contracts (see iaop-event-contracts)
- Transactional outbox/inbox for reliable event publication
- PostgreSQL as the system of record where applicable
- Identity and access via Keycloak
- Workflow orchestration via Flowable BPM (see iaop-workflow-service)
- Observability via OpenTelemetry

## Responsibilities

- Integration test harnesses
- Testcontainers helpers
- Contract test utilities
- Shared test fixtures (non-domain)

## Non-Responsibilities

- Domain test logic per service
- Production code
- Business fixtures with domain semantics

## Integration

### Upstream (callers / producers toward this service)

- (none)

### Downstream (services this depends on)

- `All IAOP services`

### Planned Integration Events

- (none planned yet)

Event schemas are governed by [iaop-event-contracts](https://github.com/AGENTAPR/iaop-event-contracts).

### Planned API Contracts

- `Crate: iaop-test-kit`

API specifications are governed by [iaop-api-contracts](https://github.com/AGENTAPR/iaop-api-contracts).

## Related Repositories

- [iaop-architecture](https://github.com/AGENTAPR/iaop-architecture) - DDD strategic design, bounded context maps, ADRs, repository landscape
- [iaop-rust-common](https://github.com/AGENTAPR/iaop-rust-common) - Shared Rust technical utilities
- [iaop-test-kit](https://github.com/AGENTAPR/iaop-test-kit) - Shared test infrastructure

## Status

**INITIALIZED - NOT IMPLEMENTED**

This repository has been bootstrapped with baseline documentation and configuration only.
No business logic or service implementation exists yet.

## Development Conventions

- Default branch: `main`
- Development branch: `development` — all changes are committed and pushed here
- Squash merges preferred; merge commits and rebases disabled
- Pull requests required before merging into `main`
- Topics: iaop, insurance, insurance-agents, enterprise-architecture, rust, testing, testcontainers, microservices
## Layout

- `crates/iaop-test-kit` — shared test helpers (ids, env guards, JSON assertions, event-envelope builders)
- No domain fixtures: service-specific test data belongs in each service repo.
