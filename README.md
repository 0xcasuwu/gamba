# 🪄 Gamba - Orbital Wand Factory

**Factory-based wand creation system using alkamist/dust tokens with cellpack architecture and boiler security patterns**

## 🌟 Overview

Gamba is an **Orbital Wand Factory** that creates individual wand NFTs through a gambling mechanism. Players send alkamist or dust tokens to improve their odds of successfully creating a wand. The factory uses boiler's proven security patterns and cellpack architecture to create individual wand NFTs from predefined templates.

## 🏗️ Factory Architecture

### Core Concept
- **Factory Pattern**: `OrbitalWandFactory` creates individual wand NFTs using cellpack calls
- **Predefined Templates**: Six wand template contracts for different rarities
- **Token Burning**: Alkamist/dust tokens are burned to improve creation odds
- **Child Registration**: Factory tracks all created wand NFTs for security

### Wand Template System
```rust
// Predefined template IDs for each wand type
const COMMON_WAND_TEMPLATE_ID: u128 = 0x1001;     // 150-170 XOR
const RARE_WAND_TEMPLATE_ID: u128 = 0x1002;       // 171-190 XOR  
const EPIC_WAND_TEMPLATE_ID: u128 = 0x1003;       // 191-210 XOR
const LEGENDARY_WAND_TEMPLATE_ID: u128 = 0x1004;  // 211-230 XOR
const MYTHIC_WAND_TEMPLATE_ID: u128 = 0x1005;     // 231-250 XOR
const COSMIC_WAND_TEMPLATE_ID: u128 = 0x1006;     // 251-255 XOR
```

## 🎮 How It Works

### Wand Creation Process
1. **Send Tokens**: Send alkamist (2:25720) or dust (2:35275) tokens to factory
2. **Calculate Bonuses**: 
   - Dust: +10 points per 1000 dust above 2000 threshold
   - Alkamist: +5 points per alkamist token
3. **Roll Calculation**: Base XOR (merkle root last byte) + bonuses
4. **Success Check**: If final XOR ≥ 150, wand creation succeeds
5. **Template Selection**: Factory selects template based on final XOR result
6. **Cellpack Creation**: Factory calls appropriate template with creation data
7. **NFT Return**: User receives individual wand NFT with unique properties

### Token Requirements
- **Alkamist**: Minimum 1 token from position 2:25720
- **Dust**: Minimum 1000 tokens from position 2:35275 or other block 2 tokens
- **Mixed**: Can send both alkamist and dust for maximum bonuses

### Success Threshold & Odds
- **Base Threshold**: 150 (values 150-255 succeed)
- **Base Win Rate**: ~41.4% (106/256 possible outcomes)
- **Enhanced Odds**: Bonuses reduce effective threshold
- **Example**: 5000 dust (+40 bonus) = ~57.0% win rate

## 🔧 Technical Implementation

### Factory Contract: `OrbitalWandFactory`

#### Core Functions
```rust
#[opcode(42)]
CastWand,                    // Main wand creation function

#[opcode(1000)]
GetData,                     // Returns SVG for latest wand

#[opcode(2000)]
GetWandCount,                // Total wands created

#[opcode(3000)]
GetAllRegisteredWands,       // All registered wand NFTs

#[opcode(3001)]
IsRegisteredWand,            // Check if wand is factory-created
```

#### Cellpack Creation Pattern
```rust
let cellpack = Cellpack {
    target: AlkaneId {
        block: WAND_TEMPLATE_BLOCK,
        tx: wand_template_id,
    },
    inputs: vec![
        0x0,                    // Initialize opcode
        wand_id,                // Wand ID
        final_xor_result as u128, // Final XOR result
        base_xor_result as u128,  // Base XOR result
        dust_bonus as u128,       // Dust bonus
        alkamist_bonus as u128,   // Alkamist bonus
        total_dust,               // Dust amount
        total_alkamist,           // Alkamist amount
        block_height as u128,     // Block height
        txid.to_byte_array()[0] as u128, // Uniqueness
    ],
};
```

### Security Patterns (Inherited from Boiler)

#### 1. Transaction Replay Protection
```rust
fn has_tx_hash(&self, txid: &Txid) -> bool
fn add_tx_hash(&self, txid: &Txid) -> Result<()>
```

#### 2. Child Registration Security
```rust
fn register_wand(&self, wand_id: &AlkaneId)
fn is_registered_wand_internal(&self, wand_id: &AlkaneId) -> bool
```

#### 3. Overflow Protection
```rust
// All arithmetic uses checked operations
let total_alkamist: u128 = alkamist_transfers.iter()
    .try_fold(0u128, |acc, t| acc.checked_add(t.value))
    .ok_or_else(|| anyhow!("Alkamist amount overflow"))?;
```

#### 4. Input Validation
```rust
fn validate_incoming_alkanes(&self) -> Result<()>
fn is_valid_alkamist_or_dust(&self, id: &AlkaneId) -> bool
```

## 🎯 Wand Types & Rarities

### By XOR Result Range
- **150-170**: Common Wands (Template 0x1001)
- **171-190**: Rare Wands (Template 0x1002)
- **191-210**: Epic Wands (Template 0x1003)
- **211-230**: Legendary Wands (Template 0x1004)
- **231-250**: Mythic Wands (Template 0x1005)
- **251-255**: Cosmic Wands (Template 0x1006) - Ultra Rare!

### Wand Properties
- **Unique ID**: Sequential factory-assigned ID
- **Template Type**: Determined by final XOR result
- **Creation Data**: Embedded alkamist/dust amounts, bonuses, block height
- **Factory Provenance**: Registered as factory child for authenticity

## 🚀 Development Setup

### Prerequisites
- Rust 1.70+
- Alkane runtime environment
- Position tokens at 2:25720 (alkamist) and 2:35275 (dust)

### Building
```bash
git clone <repository>
cd gamba
cargo build --release
```

### Testing (Indexer-Based Only)
```bash
# Run comprehensive integration tests
cargo test --test orbital_wand_integration_test

# All tests use index_block methods for proper indexer testing
```

### Build System
The build system generates real WASM bytecode:
- `build.rs` compiles contracts to WASM
- Generated files: `dust_swap.wasm`, `orbital_wand_factory.wasm`
- Test helpers: `src/tests/std/*_build.rs` use `include_bytes!` for real bytecode

## 📁 Project Structure

```
gamba/
├── src/
│   ├── lib.rs                          # Module exports
│   ├── orbital_wand.rs                 # OrbitalWandFactory implementation
│   ├── probability.rs                  # Probability calculations
│   ├── wand_svg.rs                     # SVG generation
│   └── tests/
│       ├── mod.rs                      # Test module declarations
│       ├── orbital_wand_integration_test.rs  # Comprehensive indexer tests
│       └── std/
│           ├── mod.rs                  # Build helper modules
│           ├── dust_swap_build.rs      # DustSwap bytecode helper
│           └── orbital_wand_build.rs   # Factory bytecode helper
├── build.rs                            # Build script for WASM generation
├── Cargo.toml                          # Dependencies
└── README.md                           # This file
```

## 🔒 Security Features

### Inherited from Boiler Vault Factory
1. **Transaction Replay Protection**: Each transaction can only be used once
2. **Child Registration**: All created wands are registered and tracked
3. **Overflow Protection**: All arithmetic uses checked operations
4. **Input Validation**: Strict token ID and value validation
5. **Initialization Protection**: `observe_initialization()` prevents re-init
6. **Detailed Error Messages**: Comprehensive error reporting

### Factory-Specific Security
1. **Template Validation**: Only calls predefined wand templates
2. **Token Burning**: Tokens are consumed regardless of success/failure
3. **Randomness**: Uses merkle root last byte for cryptographic randomness
4. **Bonus Caps**: Dust/alkamist bonuses are capped to prevent overflow

## 🎲 Probability & Economics

### Base Mechanics
- **Randomness Source**: Last byte of merkle root (0-255)
- **Success Threshold**: 150 (41.4% base win rate)
- **Token Burning**: All sent tokens are burned (deflationary)

### Bonus System
```rust
// Dust bonus: +10 per 1000 dust above 2000 threshold
let dust_bonus = if dust_amount >= 2000 {
    std::cmp::min(((dust_amount - 2000) / 1000) * 10, 255) as u8
} else { 0 };

// Alkamist bonus: +5 per alkamist token
let alkamist_bonus = std::cmp::min(alkamist_amount * 5, 255) as u8;
```

### Strategy Examples
- **1000 dust**: 41.4% win rate (no bonus)
- **3000 dust**: 45.3% win rate (+10 bonus)
- **5000 dust**: 49.2% win rate (+20 bonus)
- **1 alkamist**: 43.4% win rate (+5 bonus)
- **5000 dust + 2 alkamist**: 53.1% win rate (+30 total bonus)

## 🔄 Integration with Ecosystem

### Required Template Contracts
The factory requires six wand template contracts deployed at:
- Block 6, TX 0x1001-0x1006 (Common through Cosmic)
- Each template must implement initialization opcode 0x0
- Templates receive creation data via cellpack inputs

### Position Token Integration
- **Alkamist Position**: Block 2, TX 25720
- **Dust Position**: Block 2, TX 35275
- **Backward Compatibility**: Accepts any block 2 tokens as dust (except alkamist)

## 🧪 Testing Architecture

### Comprehensive Integration Tests
- **`test_orbital_wand_comprehensive_integration()`**: Full system test
- **`test_orbital_wand_edge_cases()`**: Edge case validation
- **All tests use `index_block()` methods**: Proper indexer testing
- **Multi-scenario testing**: Different dust/alkamist combinations
- **State verification**: Before/after state comparison
- **Trace analysis**: Comprehensive execution trace capture

### Test Coverage
- Contract deployment and initialization
- Position token conversion flows
- Gambling mechanics with various stakes
- SVG generation and metadata
- Statistical analysis and win rate verification
- Edge cases (zero dust, maximum dust, invalid opcodes)

## 🚀 Deployment Guide

### 1. Template Deployment
Deploy six wand template contracts to block 6, TX 0x1001-0x1006

### 2. Factory Deployment
Deploy `OrbitalWandFactory` contract

### 3. Initialization
Call initialize opcode (0x0) on factory

### 4. Verification
- Test wand creation with various token amounts
- Verify child registration works correctly
- Confirm SVG generation functions properly

## 📊 Monitoring & Analytics

### Factory Statistics
- Total wands created: `GetWandCount()`
- Win rate tracking: `GetWinRate()`
- Token consumption: `GetTotalDustConsumed()`, `GetTotalAlkamistConsumed()`
- Registered wands: `GetAllRegisteredWands()`

### Performance Metrics
- Average gas per wand creation
- Template distribution (Common vs Rare vs Epic, etc.)
- User behavior patterns (dust vs alkamist preferences)

## 🔮 Future Enhancements

### Template System Expansion
- Additional wand types and rarities
- Seasonal or event-specific templates
- Community-created template submissions

### Enhanced Mechanics
- Multi-token staking (combine different position types)
- Time-based bonuses or penalties
- Wand utility in other contracts

### Economic Features
- Wand marketplace integration
- Staking rewards for wand holders
- Governance tokens for template approval

## ⚠️ Important Notes for Next LLM

### Critical Architecture Points
1. **Factory Pattern**: This is NOT a simple gambling contract - it's a factory that creates individual NFTs
2. **Cellpack Usage**: Uses boiler's cellpack pattern to call template contracts
3. **Child Registration**: MUST register all created wands for security
4. **Token Burning**: Tokens are consumed regardless of outcome (not returned)
5. **Template Dependencies**: Requires six template contracts to be deployed first

### Security Considerations
1. **Boiler Inheritance**: Security patterns are inherited from boiler vault factory
2. **Position Validation**: Only accepts specific alkamist/dust positions
3. **Overflow Protection**: All arithmetic must use checked operations
4. **Replay Protection**: Transaction hash tracking is critical

### Testing Requirements
1. **Indexer-Only**: All tests MUST use `index_block()` methods
2. **Integration Focus**: No unit tests - only comprehensive integration tests
3. **Real Bytecode**: Build system generates actual WASM, no placeholders

### Build System
1. **WASM Generation**: `build.rs` compiles to real bytecode
2. **Include Bytes**: Test helpers use `include_bytes!` for generated WASM
3. **Template Coordination**: Factory and templates must be built together

---

**Built with 🏗️ Factory Pattern + 🔒 Boiler Security + 🎲 Provable Randomness**

*May your tokens forge legendary wands!* 🪄✨🔥