# Older Examples (Miden Assembly 0.7)

This directory contains examples that were written for Miden Assembly version 0.7 and need to be ported to work with the current version.

## Status
These examples do not compile or run with the current Miden VM version due to compatibility issues.

## Examples and Their Issues

### game_of_life_4x4.masm
**Issue:** Stack overflow due to complex memory operations

### matrix_multiplication.masm  
**Issue:** Requires porting of u64 operations and complex matrix algorithms

### proof_of_location.masm
**Issue:** Requires porting of trigonometric operations and distance calculations

### greatest_common_divisor.masm
**Issue:** Requires porting of division/modulo operations and stack management

## Current Working Examples
See the parent directory for examples that have been successfully ported to work with the current Miden VM version.