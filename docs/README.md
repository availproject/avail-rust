# Avail Rust Documentation

This directory contains guides aimed at library users. Pick the topic that fits
your workflow:

- [Basic Extrinsic Submission](basic_submission.md) – end-to-end walkthrough of
  building, signing, submitting, and tracking a transaction, including common
  pitfalls.
- [Retry Mechanism](retry_mechanism.md) – explains how global and per-call retry
  settings interact and which errors are retried.
- [Failure Modes](failure_modes.md) – catalogues why different interfaces may
  fail (network, decoding, runtime support, etc.).
- [Receipt Accuracy](receipt_accuracy.md) – details the assumptions behind
  `SubmittedTransaction::receipt()` and when it can return `None` or error.
- [Subscriptions](subscriptions.md) – shows how to configure and consume block,
  extrinsic, and justification subscriptions.
- [Block API](block_api.md) – walks through block helpers (`BlockApi`,
  `BlockWith*`, `BlockEvents`) and how to inspect block data.
- [Error Handling](error_handling.md) – outlines the SDK error types and how to
  integrate them with your application's error handling strategy.
- [Extrinsic Types](extrinsic_types.md) – explains raw, decoded, and signed
  extrinsic representations available in the SDK.

Contributions are welcome—add new guides here if you cover additional workflows
or deep dives.
