# 🎲 Gamba - Orbital Wand Gambling Contract

**Position token staking meets cryptographic gambling with dust-enhanced odds and beautiful NFT rewards**

## 🌟 Overview

Gamba is an innovative gambling contract that combines position token staking with dust-enhanced odds and stunning visual rewards. Players stake position tokens along with dust to participate in a provably fair game where higher dust amounts improve their chances. Winners receive beautiful, animated orbital wand SVG NFTs!

## 🎮 How It Works

### The Enhanced Gamble
1. **Stake**: Send a position token + at least 1000 dust to the contract
2. **Dust Bonus**: For every 1000 dust above 2000, get +10 XOR bonus points
3. **Randomness**: Contract XORs the last byte of the current block's merkle root with your transaction ID's last byte
4. **Enhanced Calculation**: Base XOR + Dust Bonus = Final XOR
5. **Outcome**: 
   - Final XOR < 141: **You lose** - your stake is consumed
   - Final XOR ≥ 141: **You win** - you receive an orbital wand NFT!

### Dust Enhancement System
- **Minimum Stake**: 1000 dust required
- **Bonus Threshold**: 2000+ dust starts giving bonuses
- **Bonus Rate**: +10 XOR points per 1000 dust increment
- **Examples**:
  - 1000 dust = +0 bonus (base odds)
  - 2000 dust = +10 bonus 
  - 3000 dust = +20 bonus
  - 5000 dust = +40 bonus
  - 10000 dust = +90 bonus

### Why It's Fair
- **Deterministic**: Uses block merkle root (miner-controlled) + transaction ID (user-controlled)
- **Transparent**: Dust bonuses are clearly calculated and displayed
- **Skill-based**: Players can improve odds by staking more dust
- **Verifiable**: All randomness sources and calculations are on-chain
- **No Oracles**: No external dependencies or trusted third parties

## 🪄 Orbital Wand NFTs

Winners receive stunning, animated SVG NFTs featuring:

### 🎨 Visual Features
- **Animated orbital wand** with magical orb, crystalline shaft, and ornate handle
- **Rotating orbital rings** around the magical core
- **Floating dust particles** with golden shimmer effects
- **Dynamic breathing animations** and magical sparkles
- **Dust enhancement display** showing your bonus contribution
- **Responsive color schemes** based on your unique final XOR result

### ⚡ Wand Properties
- **7 Wand Types**: Stellar, Nebula, Quantum, Cosmic, Void, Plasma, Orbital
- **6 Power Levels**: Apprentice → Adept → Expert → Master → Grandmaster → Cosmic
- **Unique Colors**: 6 different magical color schemes
- **Dust Enhancement**: Visual representation of your dust bonus
- **Cryptographic Proof**: Shows position token, dust amount, base XOR, bonus, and final result

### 🏆 Power Level Tiers (Based on Final XOR)
- **141-160**: Apprentice Wand
- **161-180**: Adept Wand  
- **181-200**: Expert Wand
- **201-220**: Master Wand
- **221-240**: Grandmaster Wand
- **241-255**: Cosmic Wand (Ultra Rare!)

## 🔧 Technical Architecture

### Core Contract: `OrbitalWand`
- **Opcode 42**: `cast_wand(position_token_id)` - Main gambling function
- **Opcode 1000**: `get_data()` - Returns SVG for latest wand
- **Opcode 2000**: `get_wand_count()` - Total wands minted
- **Opcode 2001**: `get_wand_list()` - All wand metadata

### Dust Swap Contract: `DustSwap`
- **Opcode 42**: `position_to_dust()` - Convert position tokens to dust
- **Opcode 69**: `dust_to_position()` - Convert dust back to position tokens
- **1:1 Ratio**: Each position token = 10,000,000,000,000 dust units

### Security Features
- **Anti-replay protection**: Each transaction can only be used once
- **Position token validation**: Only valid position tokens accepted
- **Dust amount verification**: Minimum 1000 dust required
- **Fair enhancement**: Dust bonuses are capped and transparent
- **Efficient storage**: 115 bytes per wand (includes dust data)

### Enhanced SVG Generation
- **Dynamic rendering** based on final XOR result
- **Dust particle animations** reflecting your stake
- **Bonus visualization** showing dust enhancement
- **Rich metadata** embedded in the artwork

## 🚀 Getting Started

### Prerequisites
- Position tokens from the position token contract
- Dust tokens (obtainable by converting position tokens)
- Alkane runtime environment

### Building
```bash
git clone https://github.com/your-org/gamba
cd gamba
cargo build --release
```

### Testing
```bash
cargo test
```

## 📊 Game Statistics & Strategy

### Base Odds (1000 dust, no bonus)
- **Win Rate**: ~44.9% (115/256 possible base XOR outcomes ≥ 141)
- **Loss Rate**: ~55.1% (141/256 possible base XOR outcomes < 141)

### Enhanced Odds Examples
- **3000 dust (+20 bonus)**: ~52.7% win rate (base XOR ≥ 121 wins)
- **5000 dust (+40 bonus)**: ~60.5% win rate (base XOR ≥ 101 wins)
- **10000 dust (+90 bonus)**: ~80.1% win rate (base XOR ≥ 51 wins)

### Strategy Tips
- **Risk vs Reward**: Higher dust = better odds but larger loss if you fail
- **Dust Management**: Convert position tokens to dust strategically
- **Timing**: Block merkle roots are unpredictable, so timing doesn't matter
- **Bankroll**: Only gamble with dust you can afford to lose

## 🎯 Usage Example

```rust
// Initialize the contracts
let gamba = OrbitalWand::new();
let dust_swap = DustSwap::new();

// Convert position token to dust
let position_token = AlkaneId { block: 2, tx: 12345 };
dust_swap.position_to_dust(); // Sends position token, receives dust

// Player stakes position token + dust for enhanced odds
let dust_amount = 5000; // +40 XOR bonus
let result = gamba.cast_wand(position_token); // Also send dust in transaction

// Check the outcome
match result {
    Ok(response) => {
        // Success! You won an orbital wand NFT
        println!("🎉 Victory! Base XOR + {} dust bonus = win!", dust_amount);
    },
    Err(e) => {
        // Your stake was consumed
        println!("💸 Base XOR + dust bonus still < 141. Better luck next time!");
    }
}
```

## 📁 Project Structure

```
gamba/
├── src/
│   ├── lib.rs              # DustSwap contract for position/dust conversion
│   ├── orbital_wand.rs     # Core gambling contract with dust bonuses
│   ├── wand_svg.rs         # Enhanced SVG generation with dust effects
│   └── test_orbital_wand.rs # Comprehensive tests including dust mechanics
├── Cargo.toml              # Project dependencies
├── build.rs                # Build script
├── ORBITAL_WAND_README.md  # Detailed technical docs
├── DEPLOYMENT.md           # Deployment guide
└── README.md               # This file
```

## 🔮 Future Enhancements

- **Dust Mining**: Earn dust through various activities
- **Wand Battles**: PvP combat system using wands with dust-based power
- **Spell Casting**: Use wands + dust to interact with other contracts
- **Wand Fusion**: Combine multiple wands + dust for upgrades
- **Seasonal Events**: Limited-time dust multipliers and special wand types
- **Dust Staking**: Earn rewards by staking dust long-term

## 🛡️ Security Considerations

- **Randomness Source**: Uses block merkle root + transaction ID
- **Replay Protection**: Transaction hash tracking prevents double-spending
- **Fair Enhancement**: Dust bonuses are mathematically transparent
- **Position Token Validation**: Only accepts valid position tokens
- **Dust Amount Verification**: Enforces minimum stake requirements
- **No Admin Keys**: Fully decentralized operation

## 📈 Economic Model

### Dust Economy
- **Position Token Conversion**: 1 position token = 10T dust units
- **Minimum Gamble**: 1000 dust (0.00001% of a position token)
- **Bonus Scaling**: Linear +10 per 1000 dust increment
- **Deflationary**: Lost stakes are burned, reducing dust supply

### Wand Value
- **Rarity**: Higher dust stakes can produce rarer wands
- **Utility**: Wands may have future utility in other contracts
- **Collectibility**: Each wand is unique with provable dust enhancement

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## ⚠️ Disclaimer

This is gambling software with enhanced mechanics. Please gamble responsibly and only with funds you can afford to lose. While dust bonuses improve your odds, losses are still possible. This is intended for entertainment purposes.

---

**Built with ❤️ on the Alkane blockchain**

*May your dust enhance your fortune!* 🎲✨💫