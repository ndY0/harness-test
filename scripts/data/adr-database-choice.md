---
type: adr
domain: global
feature: none
status: active
date: 2025-01-10
superseded_by: none
superseded_date: none
---
# ADR-001: Use PostgreSQL as the Primary Database

## Context

The platform requires a relational database to store structured business data
including merchants, payment intents, user accounts, and audit logs. We evaluated
three candidates: PostgreSQL, MySQL, and CockroachDB. The evaluation criteria
included transaction support, JSON handling, operational maturity, and ecosystem
compatibility with our Rust-based backend.

## Decision

We will use PostgreSQL as the primary database for all services. PostgreSQL's
native UUID support, robust JSONB indexing, and mature Rust driver ecosystem
(sqlx with compile-time query verification) make it the best fit. CockroachDB was
rejected because its serializable isolation model introduces latency that exceeds
our p95 latency budget of 200ms. MySQL was rejected due to weaker JSON support
and the lack of a mature compile-time-checked Rust driver.

## Consequences

Positive consequences: Compile-time SQL verification via sqlx eliminates an
entire class of runtime errors. JSONB columns allow flexible schema evolution
for merchant configuration without costly migrations. The PostgreSQL ecosystem
provides battle-tested tooling for backup, replication, and monitoring.

Negative consequences: The team must maintain PostgreSQL-specific SQL rather than
using a portable subset. Some engineers are more familiar with MySQL and will
require ramp-up time. Operational complexity increases slightly because we now
need to manage PostgreSQL alongside our existing Redis and Kafka infrastructure.
