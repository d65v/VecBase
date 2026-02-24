# VecBase — Common Database Layer

This directory contains the common storage abstractions used by VecBase.

## Contents

- Flat-file bincode serialization helpers
- In-memory store trait definitions
- Shared types for record storage

## Planned

- WAL (Write-Ahead Log) for crash recovery
- Snapshot / checkpoint support
- Index persistence format (`.vbi` files)
