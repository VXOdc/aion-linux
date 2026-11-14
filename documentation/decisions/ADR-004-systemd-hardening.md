# ADR-004: systemd sandboxing for AION services

Date: 2026-11-13 · Status: accepted

Every service shipped by AION must ship with systemd hardening
directives (NoNewPrivileges, ProtectSystem, ProtectHome, PrivateTmp,
minimal CapabilityBoundingSet). aiond is the reference
implementation: see services/aiond.service.
