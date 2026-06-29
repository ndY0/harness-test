---
type: spec
domain: legacy
feature: old-system
status: superseded
date: 2024-01-01
superseded_by: none
superseded_date: 2024-06-01
---
# Legacy Batch Processing System

## Overview

This specification describes the original batch payment processing system that
was used before the real-time payment gateway was introduced. The batch system
collected payment requests throughout the day and processed them in a nightly
batch run at 02:00 UTC.

## Batch Processing Pipeline

The pipeline read payment requests from the `pending_charges` MySQL table,
transformed them into ISO 20022 payment messages, and submitted them to the
banking gateway via SFTP. Results were parsed from the response files and
written back to the `charge_results` table. The entire process took approximately
45 minutes for a typical daily volume of 50,000 transactions.

## Limitations

This system was superseded because it does not support real-time payment
confirmation, cannot handle high transaction volumes during peak periods, and
requires manual intervention when the SFTP connection fails. Merchants
experienced up to 24-hour delays between payment submission and confirmation.

## Migration

All merchants have been migrated to the real-time payment gateway as of
June 2024. This specification is retained for historical reference only.
The `pending_charges` and `charge_results` tables are scheduled for removal
in Q3 2025 after the mandatory 12-month data retention period.
