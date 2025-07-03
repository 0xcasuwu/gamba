# Gamba Architecture Guide

A comprehensive analysis of the Gamba wand system based on learnings from the Boiler project, focusing on the Factory → Token pattern, orbital mechanics, and architectural decisions.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture Patterns](#architecture-patterns)
3. [Factory → Token Relationship](#factory--token-relationship)
4. [Orbital vs Vault Mechanics](#orbital-vs-vault-mechanics)
5. [Testing Framework](#testing-framework)
6. [Build System](#build-system)
7. [Key Design Decisions](#key-design-decisions)
8. [Implementation Strategy](#implementation-strategy)

## Project Overview

Gamba is a sophisticated blockchain application built on the Alkanes framework, implementing a probabilistic NFT creation system with XOR-based randomness. The project follows architectural patterns learned from the Boiler project while implementing its own unique "orbital" mechanics.

### Core Components

1. **Wand Factory System**
   - **Purpose**: Probabilistic wand creation using XOR-based randomness with DUST token enhancement
   - **Components**: Factory contract + Individual wand tokens
   - **Mechanics**: XOR calculation from blockchain data (txid ⊕ merkle_root) + DUST bonus
   - **Success Model**: Threshold-based (XOR result > 144) determines successful "orbital" creation

2. **Orbital Mechanics (vs Boiler's Vault)**
   - **Purpose**: Create unique collectible tokens with provable rarity
   - **Components**: Factory registry + Individual wand tokens with full forge history
   - **Mechanics**: One-time creation event with permanent immutable state

## Architecture Patterns

### Factory → Token Relationship

Following Boiler's proven security pattern:

```
┌─────────────────┐    creates &     ┌─────────────────┐
│   Wand Factory  │    registers     │   Wand Token    │
│   Contract      │ ───────────────> │   Contract      │
│                 │                  │                 │
│ - Registry      │                  │ - Factory Ref   │
│ - XOR Logic     │                  │ - Forge Data    │
│ - DUST Logic    │                  │ - Authentication│
│ - Success Rate  │                  │ - SVG Generation│
└─────────────────┘                  └─────────────────┘
```

**Key Benefits (learned from Boiler):**
- **Security**: Only factory-created tokens are registered
- **Verification**: O(1) child verification via registry
- **Authentication**: Tokens store factory reference for validation
- **Enumeration**: Centralized lists for iteration support
- **Spoofing Prevention**: Registry pattern prevents malicious contracts

### Message Dispatch Pattern

```rust
#[derive(MessageDispatch)]
enum WandFactoryMessage {
    #[opcode(0)]
    Initialize,
    
    #[opcode(1)]
    ForgeWand,
    
    #[opcode(10)]
    #[returns(u128)]
    GetSuccessCount,
    
    #[opcode(20)]
    #[returns(bool)]
    IsRegisteredChild { child_id: AlkaneId },
}
```

### Storage Architecture

Following Boiler's StoragePointer pattern:

```rust
// Efficient key-value storage using StoragePointer
fn success_count_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/success_count")
}

fn is_registered_child_internal(&self, child_id: &AlkaneId) -> bool {
    let key = format!("/registered_children/{}_{}", child_id.block, child_id.tx).into_bytes();
    let bytes = self.load(key);
    !bytes.is_empty() && bytes[0] == 1
}
```

## Factory → Token Relationship

### Gamba's Implementation

**Factory Responsibilities:**
- **XOR Calculation**: Deterministic randomness using blockchain data
- **DUST Integration**: Token validation and consumption for bonuses
- **Registry Management**: Track all created wand tokens
- **Success/Failure Tracking**: Statistics for forge attempts
- **Child Authentication**: Verify wand tokens are factory-created

**Token Responsibilities:**
- **Forge History**: Store complete creation context and randomness data
- **Authentication**: Validate against factory registration
- **Metadata**: Dynamic SVG generation based on forge results
- **Immutable State**: Permanent record of creation event

### Security Model (from Boiler)

```rust
fn authenticate_wand(&self, context: &Context) -> Result<()> {
    // SECURITY CRITICAL: Verify this wand token was created by US FIRST
    if !self.is_registered_child_internal(&transfer.id) {
        return Err(anyhow!(
            "Wand token not our registered child - potential spoofing attack"
        ));
    }
    
    // SECONDARY: Query wand for additional validation
    match self.get_wand_details(&transfer.id) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Registered wand failed validation: {}", e))
    }
}
```

## Orbital vs Vault Mechanics

### Key Architectural Difference

**Boiler (Vault Model):**
- **Deposit/Withdraw**: Users can add/remove assets over time
- **Reward Accrual**: Continuous time-based rewards via MasterChef algorithm
- **Position Tracking**: Mutable state with reward debt calculations
- **Temporal Caps**: Time-limited reward periods

**Gamba (Orbital Model):**
- **One-Time Creation**: Single forge event creates permanent token
- **Probability-Based**: Success determined by XOR threshold (>144)
- **Immutable History**: Complete forge context permanently stored
- **Rarity Mechanics**: DUST enhances probability but doesn't guarantee success

### Orbital Creation Process

```rust
fn forge_wand(&self) -> Result<CallResponse> {
    // 1. Validate DUST input and calculate bonus
    let dust_amount = self.get_dust_input_amount()?;
    let dust_bonus = ((dust_amount / 1000) * 5).min(255) as u8;
    
    // 2. Calculate XOR from blockchain data
    let base_xor = self.calculate_base_xor()?;
    let final_result = base_xor.saturating_add(dust_bonus);
    
    // 3. Success threshold determines orbital creation
    if final_result > 144 {
        // Successful orbital - create wand token with full context
        let wand_token = self.create_wand_token(
            dust_amount, base_xor, dust_bonus, final_result
        )?;
        self.register_child(&wand_token.id);
        self.increment_successful_forges();
    } else {
        // Failed orbital - DUST consumed but no wand created
        self.increment_failed_forges();
    }
    
    // DUST is always consumed (key economic mechanic)
    self.consume_dust_tokens(dust_amount)?;
    Ok(response)
}
```

### Immutable Forge Context

Each successful wand token permanently stores:

```rust
struct ForgeContext {
    wand_id: u128,           // Unique sequential ID
    dust_amount: u128,       // DUST tokens consumed
    base_xor: u8,           // Raw blockchain randomness
    dust_bonus: u8,         // Enhancement from DUST
    final_result: u8,       // Combined result determining success
    creation_block: u128,   // Block height of creation
    txid_hash: [u8; 32],    // Transaction ID used for XOR
    merkle_root: [u8; 32],  // Merkle root used for XOR
    dust_token_id: AlkaneId, // Reference to DUST token consumed
}
```

## Testing Framework

### Following Boiler's Proven Patterns

**Core Testing Infrastructure:**
```rust
// Blockchain simulation via indexer integration
use alkanes::view;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::clear;

pub struct WandTestingSetup {
    pub factory_block: u32,
    pub factory_tx: u32,
    pub dust_token_id: AlkaneId,
}

pub struct ForgeAnalysis {
    pub success: bool,
    pub wand_tokens: Vec<AlkaneTransfer>,
    pub dust_consumed: bool,
    pub xor_calculation: u8,
    pub dust_bonus: u8,
    pub final_result: u8,
}
```

**Test Categories (adapted from Boiler):**
1. **Factory Operations**: Initialization, forge validation, state management
2. **XOR Calculations**: Mathematical precision, determinism verification
3. **DUST Integration**: Token validation, bonus calculation, consumption
4. **Token Creation**: Registry management, state storage, authentication
5. **Edge Cases**: Zero amounts, wrong tokens, overflow protection
6. **End-to-End**: Complete workflows, success rates, trace verification

**Trace Analysis Pattern:**
```rust
pub fn analyze_forge_trace(outpoint: &OutPoint) -> Result<ForgeAnalysis> {
    let trace_data = view::trace(outpoint)?;
    let trace_result: Trace = AlkanesTrace::parse_from_bytes(trace_data)?.into();
    
    let trace_str = format!("{:?}", trace_result.0.lock().unwrap());
    let success = trace_str.contains("ReturnContext");
    
    // Extract XOR calculations, DUST consumption, token creation
    // Verify mathematical precision and registry updates
}
```

## Build System

### Following Boiler's Architecture

**Custom Build Script Integration:**
```rust
// build.rs automatically:
// 1. Discovers contracts in alkanes/*/ 
// 2. Compiles to WASM
// 3. Compresses with gzip
// 4. Generates hex modules in src/tests/std/
// 5. Updates mod.rs

// Generated artifacts:
// src/tests/std/wand_factory_build.rs
// src/tests/std/wand_token_build.rs
// src/tests/std/mod.rs
```

**Build Flow:**
```
alkanes/
├── wand-factory/     ┐
└── wand-token/       │ cargo build --release --target wasm32-unknown-unknown
                      ↓
                   WASM binaries
                      ↓
                   gzip compression
                      ↓
                   hex encoding
                      ↓
                 src/tests/std/
                 ├── wand_factory_build.rs
                 ├── wand_token_build.rs
                 └── mod.rs
```

## Key Design Decisions

### 1. XOR Source Selection
- **Decision**: Use last bytes of txid ⊕ merkle_root
- **Rationale**: Even distribution, unmanipulable by users
- **Implementation**: `txid_bytes[31] ^ merkle_bytes[31]`

### 2. DUST Bonus Formula  
- **Decision**: Linear scaling with 1000-unit increments
- **Formula**: `(dust_amount / 1000) * 5` capped at 255
- **Benefits**: Predictable enhancement, economic incentive

### 3. Success Threshold
- **Decision**: XOR result > 144 (56.25% base success rate)
- **Rationale**: Balanced probability allowing DUST to meaningfully impact outcomes
- **Economics**: DUST consumption regardless of success creates deflationary pressure

### 4. Registry Pattern (from Boiler)
- **Decision**: O(1) verification with enumeration support
- **Implementation**: Individual keys + centralized list  
- **Benefits**: Fast lookups, iteration capability, security

### 5. Immutable Orbitals
- **Decision**: Complete forge context permanently stored
- **Rationale**: Provable rarity, historical verification, collectible value
- **Implementation**: All randomness inputs and calculations preserved

## Implementation Strategy

### Phase 1: Core Infrastructure (✓ COMPLETE)
- [x] Build system following Boiler pattern
- [x] Hex file generation in src/tests/std/
- [x] Factory → Token architecture
- [x] Message dispatch patterns
- [x] Storage pointer implementation

### Phase 2: Factory Logic Implementation
- [ ] XOR calculation using blockchain data
- [ ] DUST token validation and consumption
- [ ] Success threshold evaluation
- [ ] Registry management
- [ ] Child token creation

### Phase 3: Token Implementation  
- [ ] Forge context storage
- [ ] Factory authentication
- [ ] SVG generation based on rarity
- [ ] Metadata generation
- [ ] Token trait implementation

### Phase 4: Testing Framework
- [ ] Test utilities following Boiler patterns
- [ ] Trace analysis implementation
- [ ] End-to-end forge testing
- [ ] Mathematical precision validation
- [ ] Security verification

### Phase 5: Integration & Optimization
- [ ] Performance optimization
- [ ] Gas usage analysis
- [ ] Error handling refinement
- [ ] Documentation completion

## Memory Bank Insights

### Critical Learnings from Boiler

1. **Security First**: Always implement registry-based child verification before any other validation
2. **Trace Analysis**: Comprehensive execution verification is essential for complex state transitions
3. **Build Automation**: Custom build scripts enable seamless development workflow
4. **Storage Patterns**: StoragePointer provides clean, efficient key-value operations
5. **Message Dispatch**: Clear opcode numbering prevents conflicts and enables clean APIs

### Gamba-Specific Considerations

1. **Economic Model**: DUST consumption regardless of success creates interesting tokenomics
2. **Provable Rarity**: Complete forge history enables verification of token legitimacy
3. **Collector Value**: Immutable creation context makes each wand unique and traceable
4. **Threshold Mechanics**: Success probability can be tuned via threshold adjustment
5. **Enhancement Model**: DUST provides meaningful but not guaranteed improvements

### Future Expansion Opportunities

1. **Multi-Token Support**: Factory could support different token types
2. **Dynamic Thresholds**: Success rates could vary based on conditions
3. **Seasonal Events**: Special DUST bonuses or threshold modifications
4. **Cross-Factory Integration**: Wands could interact with other alkane systems
5. **Governance Integration**: Community-driven parameter adjustments

---

This architecture guide serves as the foundation for Gamba development, incorporating proven patterns from Boiler while implementing novel orbital mechanics suitable for collectible token creation.