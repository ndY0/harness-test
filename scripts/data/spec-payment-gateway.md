---
type: spec
domain: payments
feature: payment-gateway
status: active
date: 2025-03-15
superseded_by: none
superseded_date: none
---
# Payment Gateway Integration

## Overview

This specification defines the integration between the platform and external payment
gateways such as Stripe, Adyen, and Braintree. The system must support multiple
payment providers concurrently and allow merchants to configure which provider they
use at onboarding time.

## Functional Requirements

The payment gateway module must provide a unified interface for creating payment
intents, processing refunds, and handling webhook callbacks. Each provider adapter
must implement the `PaymentProvider` trait, which exposes `create_intent()`,
`capture_intent()`, and `refund_intent()` methods. The routing layer dispatches
calls to the correct adapter based on the merchant configuration stored in PostgreSQL.

## Non-Functional Requirements

The system must process payment intent creation within 200ms at the p95 percentile
under a load of 1000 requests per second. Webhook processing must be idempotent,
meaning duplicate webhook deliveries for the same event must not result in
duplicate state transitions or double-charges.

## API Endpoints

The gateway exposes a REST API at `/api/v1/payments` with the following resources:
`POST /api/v1/payments/intents` for creating a new payment intent,
`POST /api/v1/payments/intents/{id}/capture` for capturing an authorized intent,
and `POST /api/v1/payments/intents/{id}/refund` for issuing a refund.

## Data Model

Each payment intent is stored in the `payment_intents` table with columns for
`id`, `merchant_id`, `amount`, `currency`, `status`, `provider`, `provider_intent_id`,
`idempotency_key`, `created_at`, and `updated_at`. The status transitions follow a
strict state machine: `pending` → `authorized` → `captured` → `refunded`.

## Testing Strategy

Unit tests must cover each provider adapter in isolation using mock HTTP servers.
Integration tests must verify the full payment flow against each provider's sandbox
environment. Contract tests must validate that the `PaymentProvider` trait is
implemented correctly by every adapter.

## Migration Plan

Existing merchants using the legacy charging module will be migrated gradually.
Each merchant record gains a `migration_batch` field, and we process one batch
per day. Rollback is possible for 30 days after migration by flipping the
`payment_backend` column back to `legacy`.
