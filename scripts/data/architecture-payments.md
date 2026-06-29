---
type: architecture
domain: payments
feature: payment-gateway
status: active
date: 2025-03-20
superseded_by: none
superseded_date: none
---
# Payments Domain Architecture

## Bounded Context

The Payments bounded context owns all payment-related business logic including
payment intent creation, capture, refund, and settlement reconciliation. It
communicates with the Orders context asynchronously via a message broker.
The Payments context does not own merchant data but reads merchant configuration
from the Merchants context via a synchronous REST call with circuit-breaking.

## Event Flow

When a payment intent is created, the Payments service emits a `PaymentIntentCreated`
event to the `payments.events` Kafka topic. Downstream consumers include the
Fraud Detection service, the Ledger service for double-entry bookkeeping, and the
Notification service for user-facing status updates. All events are persisted to
an outbox table before publishing, ensuring at-least-once delivery semantics.

## Data Model

The Payments bounded context owns the `payment_intents`, `refunds`, and
`settlement_batches` tables. Each table uses UUIDv7 primary keys for
time-sortable identifiers. The `payment_intents` table is the primary aggregate
root; refunds are child entities that reference their parent intent via
`payment_intent_id` with an ON DELETE RESTRICT constraint.

## Integration Patterns

External payment providers are integrated via the Adapter pattern. Each provider
adapter implements the `PaymentProvider` trait and is registered in a provider
registry at application startup. The registry dispatches by `provider_name` at
runtime. Adapters use the Provider Configuration stored in the
`payment_provider_configs` table, which holds API keys encrypted at rest using
AWS KMS envelope encryption.

## Resilience Strategy

All outbound calls to external providers use retry with exponential backoff
(max 5 retries over 30 seconds) and circuit breaking (5 consecutive failures
opens the circuit for 60 seconds). Inbound idempotency is guaranteed by the
`idempotency_key` field on payment intents; duplicate requests with the same
key return the existing result rather than creating a duplicate resource.

## Deployment

The Payments service is deployed as a containerized Rust application on
Kubernetes with a minimum of 3 replicas spread across availability zones.
PostgreSQL is used as the primary data store, with read replicas for
reporting queries. Kafka is used for asynchronous inter-service communication.
