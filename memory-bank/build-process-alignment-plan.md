# Build Process Alignment Plan: Gamba → Boiler

## Overview

This document outlines the detailed plan to align gamba's build process with boiler's approach, enabling fresh WASM compiles and proper testing iterations. The goal is to transition from gamba's custom build script approach to boiler's modular workspace structure.

## Current State Analysis

### Gamba's Current Build Process
- **Custom build.rs script** that compiles WASM and generates hex-encoded Rust files
- **Precompiled modules** stored as hex strings in `src/precompiled/`
- **Manual dependency management** through build script
- **Limited iteration speed** due to precompiled approach

### Boiler's Target Build Process
- **Cargo workspace** with modular crate structure
- **Standard Cargo build system** with proper dependency management
- **Fresh WASM compilation** on each build
- **Comprehensive testing framework** with integration tests

## Phase 1: Workspace Structure Migration

### Step 1: Create Cargo Workspace
**File**: `Cargo.toml` (root)

```toml
[workspace]
resolver = "2"
members = [
    "contracts/factory",
    "contracts/coupon-template", 
    "contracts/free-mint",
    "contracts/auth-token",
    "crates/alkanes-runtime",
    "crates/alkanes-support",
    "crates/alkanes-macros",
    "crates/metashrew-support",
    "crates/protorune-support",
    "tests"
]

[workspace.dependencies]
alkanes-runtime = { path = "crates/alkanes-runtime" }
alkanes-support = { path = "crates/alkanes-support" }
alkanes-macros = { path = "crates/alkanes-macros" }
metashrew-support = { path = "crates/metashrew-support" }
protorune-support = { path = "crates/protorune-support" }
anyhow = { version = "1.0", features = ["backtrace"] }
bitcoin = { version = "0.32.4", features = ["rand"] }
wasm-bindgen = "0.2.89"
wasm-bindgen-test = "0.3.39"
```

### Step 2: Migrate Contract Structure
**Directory Structure**:
```
gamba/
├── contracts/
│   ├── factory/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── coupon-template/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── free-mint/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── auth-token/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── crates/
│   ├── alkanes-runtime/
│   ├── alkanes-support/
│   ├── alkanes-macros/
│   ├── metashrew-support/
│   └── protorune-support/
└── tests/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── integration_test.rs
        ├── factory_test.rs
        └── deposit_validation_test.rs
```

### Step 3: Contract Cargo.toml Files

**Factory Contract** (`contracts/factory/Cargo.toml`):
```toml
[package]
name = "factory"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
alkanes-runtime = { workspace = true }
alkanes-support = { workspace = true }
alkanes-macros = { workspace = true }
anyhow = { workspace = true }
bitcoin = { workspace = true }
```

**Coupon Template Contract** (`contracts/coupon-template/Cargo.toml`):
```toml
[package]
name = "coupon-template"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
alkanes-runtime = { workspace = true }
alkanes-support = { workspace = true }
alkanes-macros = { workspace = true }
anyhow = { workspace = true }
bitcoin = { workspace = true }
```

### Step 4: Test Framework Setup
**Test Cargo.toml** (`tests/Cargo.toml`):
```toml
[package]
name = "gamba-tests"
version = "0.1.0"
edition = "2021"

[dependencies]
factory = { path = "../contracts/factory" }
coupon-template = { path = "../contracts/coupon-template" }
free-mint = { path = "../contracts/free-mint" }
auth-token = { path = "../contracts/auth-token" }
alkanes-runtime = { workspace = true }
alkanes-support = { workspace = true }
metashrew-support = { workspace = true }
protorune-support = { workspace = true }
anyhow = { workspace = true }
bitcoin = { workspace = true }
wasm-bindgen-test = { workspace = true }
```

## Phase 2: Build System Implementation

### Step 1: Remove Custom Build Script
**Action**: Delete or archive `build.rs` and `src/precompiled/` directory

### Step 2: Implement Standard Build Commands
**Build Scripts** (`scripts/build.sh`):
```bash
#!/bin/bash

# Build all contracts
echo "Building contracts..."
cargo build --target wasm32-unknown-unknown --release

# Build tests
echo "Building tests..."
cargo build --package gamba-tests

# Run tests
echo "Running tests..."
cargo test --package gamba-tests
```

### Step 3: WASM Output Management
**Directory Structure**:
```
target/
└── wasm32-unknown-unknown/
    └── release/
        ├── factory.wasm
        ├── coupon_template.wasm
        ├── free_mint.wasm
        └── auth_token.wasm
```

## Phase 3: Testing Framework Migration

### Step 1: Integration Test Structure
**File**: `tests/src/integration_test.rs`

```rust
use factory::FactoryAlkane;
use coupon_template::CouponTemplateAlkane;
use free_mint::FreeMintAlkane;
use auth_token::AuthTokenAlkane;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use anyhow::Result;

#[test]
fn test_full_deposit_validation_flow() -> Result<()> {
    // Step 1: Deploy and initialize contracts
    let (factory_id, free_mint_id, auth_token_id) = setup_contracts()?;
    
    // Step 2: Mint tokens from free-mint
    let minted_tokens = mint_tokens(&free_mint_id)?;
    
    // Step 3: Validate deposit into factory
    let deposit_result = validate_deposit(&factory_id, &minted_tokens)?;
    
    // Step 4: Verify coupon creation
    let coupon = verify_coupon_creation(&factory_id, &deposit_result)?;
    
    // Step 5: Test validation failures
    test_validation_failures(&factory_id)?;
    
    Ok(())
}

fn setup_contracts() -> Result<(AlkaneId, AlkaneId, AlkaneId)> {
    // Implementation following boiler patterns
    todo!("Implement contract setup")
}

fn mint_tokens(free_mint_id: &AlkaneId) -> Result<AlkaneTransfer> {
    // Implementation following boiler patterns
    todo!("Implement token minting")
}

fn validate_deposit(factory_id: &AlkaneId, tokens: &AlkaneTransfer) -> Result<CallResponse> {
    // Implementation following boiler patterns
    todo!("Implement deposit validation")
}

fn verify_coupon_creation(factory_id: &AlkaneId, deposit_result: &CallResponse) -> Result<AlkaneTransfer> {
    // Implementation following boiler patterns
    todo!("Implement coupon verification")
}

fn test_validation_failures(factory_id: &AlkaneId) -> Result<()> {
    // Test various validation failure scenarios
    todo!("Implement validation failure tests")
}
```

### Step 2: Deposit Validation Test
**File**: `tests/src/deposit_validation_test.rs`

```rust
use factory::FactoryAlkane;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use anyhow::Result;

#[test]
fn test_deposit_validation_success() -> Result<()> {
    let factory = FactoryAlkane::default();
    let factory_id = AlkaneId { block: 1, tx: 1 };
    
    // Test valid deposit
    let valid_tokens = AlkaneTransfer {
        id: AlkaneId { block: 2, tx: 1 }, // Valid token from free-mint
        value: 1000, // Above minimum stake
    };
    
    let result = factory.validate_deposit(&valid_tokens)?;
    assert!(result.is_ok());
    
    Ok(())
}

#[test]
fn test_deposit_validation_failures() -> Result<()> {
    let factory = FactoryAlkane::default();
    
    // Test invalid token type
    let invalid_tokens = AlkaneTransfer {
        id: AlkaneId { block: 999, tx: 999 }, // Invalid token
        value: 1000,
    };
    
    let result = factory.validate_deposit(&invalid_tokens);
    assert!(result.is_err());
    
    // Test insufficient stake
    let low_stake = AlkaneTransfer {
        id: AlkaneId { block: 2, tx: 1 },
        value: 100, // Below minimum
    };
    
    let result = factory.validate_deposit(&low_stake);
    assert!(result.is_err());
    
    Ok(())
}
```

## Phase 4: Migration Scripts

### Step 1: Migration Helper Script
**File**: `scripts/migrate_to_workspace.sh`

```bash
#!/bin/bash

echo "Migrating gamba to workspace structure..."

# Create new directory structure
mkdir -p contracts/{factory,coupon-template,free-mint,auth-token}/src
mkdir -p crates/{alkanes-runtime,alkanes-support,alkanes-macros,metashrew-support,protorune-support}/src
mkdir -p tests/src

# Move existing contract code
echo "Moving contract code..."
cp src/alkanes/factory/src/lib.rs contracts/factory/src/
cp src/alkanes/coupon-template/src/lib.rs contracts/coupon-template/src/
cp src/alkanes/free-mint/src/lib.rs contracts/free-mint/src/

# Create Cargo.toml files
echo "Creating Cargo.toml files..."
# (Cargo.toml content from Step 3 above)

# Archive old build system
echo "Archiving old build system..."
mkdir -p archive
mv build.rs archive/
mv src/precompiled archive/

echo "Migration complete!"
echo "Next steps:"
echo "1. Update dependencies in Cargo.toml files"
echo "2. Fix import paths in contract code"
echo "3. Run 'cargo build' to test workspace"
echo "4. Run 'cargo test' to verify tests"
```

### Step 2: Dependency Update Script
**File**: `scripts/update_dependencies.sh`

```bash
#!/bin/bash

echo "Updating dependencies..."

# Update all Cargo.toml files with workspace dependencies
find . -name "Cargo.toml" -exec sed -i '' 's/path = "\.\.\/\.\.\/\.\.\/crates\/alkanes-runtime"/{ workspace = true }/g' {} \;
find . -name "Cargo.toml" -exec sed -i '' 's/path = "\.\.\/\.\.\/\.\.\/crates\/alkanes-support"/{ workspace = true }/g' {} \;

echo "Dependencies updated!"
```

## Phase 5: Validation Implementation

### Step 1: Factory Deposit Validation
**File**: `contracts/factory/src/lib.rs`

```rust
impl FactoryAlkane {
    fn validate_deposit(&self, tokens: &AlkaneTransfer) -> Result<()> {
        // Validate token type
        if !self.is_valid_stake_token(&tokens.id) {
            return Err(anyhow!("Invalid token type for staking"));
        }
        
        // Validate minimum stake
        if tokens.value < self.minimum_stake_amount() {
            return Err(anyhow!("Insufficient stake amount. Minimum: {}", self.minimum_stake_amount()));
        }
        
        // Validate factory initialization
        if !self.is_initialized() {
            return Err(anyhow!("Factory not initialized"));
        }
        
        Ok(())
    }
    
    fn is_valid_stake_token(&self, token_id: &AlkaneId) -> bool {
        // Check if token is from initialized free-mint contract
        self.initialized_tokens.contains(token_id)
    }
    
    fn minimum_stake_amount(&self) -> u128 {
        self.load_u128("/minimum_stake_amount").unwrap_or(1000)
    }
    
    fn is_initialized(&self) -> bool {
        !self.load("/initialized".as_bytes().to_vec()).is_empty()
    }
}
```

### Step 2: Integration with Create Coupon
**File**: `contracts/factory/src/lib.rs`

```rust
impl FactoryAlkane {
    fn create_coupon(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        // Validate incoming tokens
        let stake_amount = self.validate_incoming_tokens(&context)?;
        
        // Calculate base XOR from blockchain data
        let base_xor = self.calculate_base_xor_internal()?;
        let stake_bonus = self.calculate_stake_bonus_internal(stake_amount)?;
        let final_result = base_xor.saturating_add(stake_bonus);

        // Check success threshold
        let success_threshold = self.success_threshold();
        if final_result > success_threshold {
            // Successful gamble - create winning coupon token
            let coupon_token = self.create_coupon_token(
                stake_amount,
                base_xor,
                stake_bonus,
                final_result,
                true, // winning coupon
            )?;

            // Register the coupon token as our child
            self.register_coupon(&coupon_token.id);

            // Increment successful coupons
            let new_successful = self.successful_coupons().checked_add(1).unwrap_or(0);
            self.set_successful_coupons(new_successful);

            // Return the coupon token to the user
            response.alkanes.0.push(coupon_token);
        } else {
            // Failed gamble - create losing coupon token
            let coupon_token = self.create_coupon_token(
                stake_amount,
                base_xor,
                stake_bonus,
                final_result,
                false, // losing coupon
            )?;

            // Register the coupon token as our child
            self.register_coupon(&coupon_token.id);

            // Increment failed coupons
            let new_failed = self.failed_coupons().checked_add(1).unwrap_or(0);
            self.set_failed_coupons(new_failed);

            // Return the coupon token to the user
            response.alkanes.0.push(coupon_token);
        }

        Ok(response)
    }
    
    fn validate_incoming_tokens(&self, context: &Context) -> Result<u128> {
        let mut total_received = 0;
        for transfer in &context.incoming_alkanes.0 {
            // Validate token type
            if !self.is_valid_stake_token(&transfer.id) {
                return Err(anyhow!("Invalid token type for staking: {:?}", transfer.id));
            }
            total_received += transfer.value;
        }
        
        if total_received == 0 {
            return Err(anyhow!("No tokens received for staking"));
        }
        
        // Validate minimum stake
        if total_received < self.minimum_stake_amount() {
            return Err(anyhow!("Insufficient stake amount. Received: {}, Minimum: {}", 
                total_received, self.minimum_stake_amount()));
        }
        
        Ok(total_received)
    }
}
```

## Implementation Timeline

### Week 1: Workspace Structure
- [ ] Create workspace Cargo.toml
- [ ] Migrate contract structure
- [ ] Set up basic crate dependencies

### Week 2: Build System
- [ ] Remove custom build script
- [ ] Implement standard build commands
- [ ] Test WASM compilation

### Week 3: Testing Framework
- [ ] Set up test framework
- [ ] Implement integration tests
- [ ] Add deposit validation tests

### Week 4: Validation Implementation
- [ ] Implement deposit validation
- [ ] Add error handling
- [ ] Test validation scenarios

### Week 5: Integration & Testing
- [ ] Full integration testing
- [ ] Performance testing
- [ ] Documentation updates

## Success Criteria

1. **Fresh WASM Compiles**: Each build generates fresh WASM files
2. **Proper Validation**: Deposit validation works correctly
3. **Comprehensive Testing**: Full test suite passes
4. **Build Speed**: Build process is fast and reliable
5. **Maintainability**: Code is easy to maintain and extend

## Risk Mitigation

1. **Backup Current System**: Archive current build system before migration
2. **Incremental Migration**: Migrate one contract at a time
3. **Continuous Testing**: Test each step of migration
4. **Rollback Plan**: Keep ability to rollback to current system

This alignment plan will enable gamba to generate fresh WASM compiles and implement proper deposit validation, achieving the goal of successful deposit into the factory of hash cash.
