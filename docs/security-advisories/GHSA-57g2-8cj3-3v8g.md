# Malformed GRIB2 data packed with complex packing can cause a decoder panic

## Summary

Applications that decode untrusted GRIB2 data using complex packing, with or without spatial differencing, may panic when the data length calculated from the packing metadata exceeds the actual length of Section 7.

## Affected usage and impact

The affected paths decode data representation templates 5.2 and 5.3 through `Grib2SubmessageDecoder::dispatch()`. Attacker-controlled messages can pass initial file parsing and decoder construction before triggering a bounds panic during decoder dispatch. Reading metadata alone does not trigger these decoding paths.

A panic does not necessarily cause denial of service. Depending on the application's panic handling and configured panic strategy (unwind or abort), a panic may interrupt a processing thread or terminate the process. This can become a denial-of-service vulnerability when an attacker can supply input to the affected operation and the resulting failure disrupts service for legitimate users. A failure confined to the attacker's request does not by itself establish service-wide denial of service; the impact depends on how the application isolates failures and maintains processing capacity.

No memory corruption, confidentiality impact, or integrity impact has been established. The confirmed failure is a Rust bounds-check panic.

## Cause

The decoder derives byte ranges from group counts, bit widths, and spatial-differencing parameters without checking every required range against the available Section 7 payload before slicing it.

## Mitigation and remediation

Upgrade the `grib` crate to version **0.20.2**, which fixes this issue.

## Version and publication status

The final version of [GHSA-57g2-8cj3-3v8g](https://github.com/noritada/grib-rs/security/advisories/GHSA-57g2-8cj3-3v8g). The fix is confirmed in `grib` version **0.20.2**. Affected versions of `grib`: `>= 0.2.0, < 0.20.2`.
