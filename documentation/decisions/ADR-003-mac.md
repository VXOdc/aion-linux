# ADR-003: AppArmor as the mandatory access control framework

Date: 2026-11-13 · Status: accepted

## Context
The PRD allows "AppArmor or SELinux". A small team cannot maintain
SELinux policy for a novel distribution; AppArmor profiles are
per-application and far cheaper to author and audit.

## Decision
Use **AppArmor**. Ship enforcing profiles for aiond, the runtimes,
and Agent Hub agents. SELinux stays out of scope for 1.0.

## Consequences
- Faster time-to-first-secure-release.
- Fewer policy-related boot failures on community hardware.
- If SELinux is ever revisited, that is a new ADR.
