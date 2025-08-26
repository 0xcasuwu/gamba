# Deployment Schedule Insights: Blocks 3, 4, 6 Pattern

## Overview
This document captures our deep insights into the deployment schedule pattern used in alkanes-based gambling systems, particularly the critical relationship between blocks 3, 4, and 6.

## The 3-4-6 Deployment Pattern

### Block 0: Template Deployment
- **Purpose**: Deploy all contract templates to the system
- **Contracts**: Free-mint, Factory, Coupon templates
- **Key Insight**: Templates are deployed but not yet instantiated

### Block 1: Free-Mint Contract Initialization
- **Target**: Block 6, TX 797 (future deployment)
- **Purpose**: Initialize the free-mint contract
- **Critical Output**: Creates auth token at Block 2, TX 2
- **Parameters**: 
  - `auth_token_units`: 1000000 (auth token supply)
  - `token_units`: 1000000 (initial token supply)
  - `value_per_mint`: 100000
  - `cap`: 1000000000
  - `name_part1`: 0x54455354 ("TEST")
  - `name_part2`: 0x434f494e ("COIN")
  - `symbol`: 0x545354 ("TST")

### Block 2: Auth Token Creation
- **Location**: Block 2, TX 2
- **Purpose**: Authorization token for factory operations
- **Critical**: Required for factory authorization

### Block 3: Factory Initialization
- **Target**: Block 4, TX 0x701
- **Purpose**: Initialize the coupon factory
- **Parameters**:
  - `success_threshold`: 144 (XOR threshold)
  - `coupon_template_id`: Block 4, TX 0x601

### Block 4: Factory Instance
- **Location**: Block 4, TX 0x701
- **Purpose**: Active factory contract
- **Dependencies**: Requires auth token from Block 2

### Block 5: Factory Authorization
- **Purpose**: Authorize factory using auth token
- **Input**: Auth token from Block 2, TX 2
- **Output**: Factory becomes operational

### Block 6: Free-Mint Instance
- **Location**: Block 6, TX 797
- **Purpose**: Active free-mint contract
- **Dependencies**: Initialized from Block 1

## Critical Insights

### 1. Authorization Chain
```
Block 1 (Init) → Block 2 (Auth Token) → Block 5 (Auth Factory) → Block 6 (Active)
```

### 2. Template vs Instance Pattern
- **Templates**: Deployed at Block 0, contain contract bytecode
- **Instances**: Created at specific blocks (4, 6), contain contract state
- **Key**: Templates must be deployed before instances can be created

### 3. Block Number Significance
- **Block 3**: Factory initialization (targets Block 4)
- **Block 4**: Factory instance location
- **Block 6**: Free-mint instance location
- **Pattern**: 3 deploys to 4, 6 is spawned at 2

### 4. Contract Compatibility Issues

#### Current Problem
The gamba project's contract templates are **fundamentally incompatible** with the current alkanes runtime:

- **"Unrecognized opcode"**: WASM contracts use opcodes that don't exist in current runtime
- **"unexpected end of file"**: Contracts not properly initialized
- **"unexpected end-of-file (at offset 0x0)"**: Contracts completely empty/corrupted

#### Root Cause
The contract templates were built with an **outdated alkanes toolchain** and need to be rebuilt with the current runtime.

### 5. Deployment Schedule Logic

#### Why Block 6 for Free-Mint?
- **Separation**: Keeps free-mint separate from factory initialization
- **Authorization**: Allows time for auth token creation and factory authorization
- **Pattern**: Follows the "3 deploys to 4, 6 targets 4 and spawns at 2" pattern

#### Why Block 4 for Factory?
- **Early Availability**: Factory needs to be ready before gambling operations
- **Template Dependency**: Depends on coupon template from Block 0
- **Authorization Ready**: Can be authorized at Block 5

### 6. Token Flow Pattern

#### Minting Flow
1. **Block 1**: Initialize free-mint contract (targets Block 6)
2. **Block 5**: Mint tokens from free-mint contract
3. **Block 6**: Free-mint contract becomes active
4. **Block 7+**: Use minted tokens for gambling

#### Gambling Flow
1. **Block 4**: Factory is ready
2. **Block 5**: Factory is authorized
3. **Block 7+**: Create coupons using minted tokens

## Implementation Requirements

### 1. Contract Template Updates
- Rebuild all WASM contracts with current alkanes toolchain
- Update contract interfaces to match current alkanes API
- Ensure opcode compatibility with current runtime

### 2. Deployment Sequence
```rust
// Block 0: Deploy templates
let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
    [free_mint_build::get_bytes(), factory_build::get_bytes(), coupon_template_build::get_bytes()].into(),
    [vec![3u128, 797u128, 101u128], vec![3u128, 0x701, 10u128], vec![3u128, 0x601, 10u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
);

// Block 1: Initialize free-mint (targets Block 6)
// Block 2: Auth token created
// Block 3: Initialize factory (targets Block 4)
// Block 4: Factory instance ready
// Block 5: Authorize factory
// Block 6: Free-mint instance ready
```

### 3. Authorization Pattern
```rust
// Factory authorization requires auth token
let auth_token_outpoint = OutPoint {
    txid: free_mint_block.txdata[0].compute_txid(),
    vout: 0,
};

// Use auth token to authorize factory
let authorize_factory_block = create_authorization_transaction(
    auth_token_outpoint,
    factory_id,
    auth_token_rune_id
);
```

## Key Takeaways

1. **Deployment Schedule is Critical**: The 3-4-6 pattern is not arbitrary but follows a specific authorization and dependency chain.

2. **Template vs Instance**: Understanding the difference between contract templates (deployed at Block 0) and contract instances (created at specific blocks) is essential.

3. **Authorization Chain**: The auth token created at Block 2 is critical for factory operations and must be properly managed.

4. **Contract Compatibility**: The current gamba contracts need to be rebuilt with the current alkanes runtime to work properly.

5. **Block Timing**: Each block serves a specific purpose in the deployment sequence and cannot be reordered without breaking the authorization chain.

## Next Steps

1. **Rebuild Contract Templates**: Update all WASM contracts to be compatible with current alkanes runtime
2. **Test Deployment Sequence**: Verify the 3-4-6 pattern works with updated contracts
3. **Implement Authorization Flow**: Ensure proper auth token handling
4. **Validate Token Flow**: Test minting and gambling operations end-to-end

This deployment schedule pattern is fundamental to the gamba system architecture and must be understood and implemented correctly for the system to function.
