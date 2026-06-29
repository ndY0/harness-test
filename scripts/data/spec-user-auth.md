---
type: spec
domain: auth
feature: user-auth
status: active
date: 2025-04-10
superseded_by: none
superseded_date: none
---
# User Authentication Service

## Overview

This specification covers the user authentication and authorization subsystem.
It describes the OAuth2-based login flow, session management with rotating refresh
tokens, and the role-based access control (RBAC) model that gates access to all
other platform services.

## OAuth2 Login Flow

Users authenticate via external identity providers (Google, GitHub, Microsoft)
using the Authorization Code Grant with PKCE. After successful authentication,
the authorization server issues a short-lived access token (15 minutes) and a
long-lived refresh token (30 days). The refresh token is stored as a SHA-256
hash in the `refresh_tokens` table alongside the user ID, device fingerprint,
and expiry timestamp.

## Session Management

Sessions are identified by the access token JWT, which contains the user ID,
roles, and a session UUID. All downstream services validate the JWT against
the public key published by the auth service at `/.well-known/jwks.json`.
Token revocation is handled by a bloom-filter-based deny list that is
replicated to every service node via Redis pub-sub.

## Password Hashing

For local account authentication (when no external IdP is configured), passwords
are hashed using Argon2id with parameters: memory=65536 KiB, iterations=3,
parallelism=4, salt length=16 bytes. Password upgrades are triggered automatically
when a user authenticates with a legacy bcrypt hash.

## RBAC Model

Roles are defined as collections of permissions. The platform ships with four
built-in roles: `admin`, `merchant_admin`, `developer`, and `viewer`. Custom
roles can be created by admin users. Permission evaluation uses a deny-override
model: if any role assigned to a user explicitly denies a permission, the user
is denied regardless of other roles.

## Security Considerations

All auth endpoints must be rate-limited to 5 requests per second per IP address.
Failed login attempts are tracked in a sliding window; after 10 failures within
15 minutes, the account is locked for 30 minutes. JWT signing keys are rotated
every 7 days, with a 14-day overlap to ensure zero-downtime key transitions.
