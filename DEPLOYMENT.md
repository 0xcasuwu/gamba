# 🚀 Gamba Deployment Guide

## Quick Start

### 1. Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install alkane CLI (if available)
# cargo install alkane-cli
```

### 2. Build the Contract
```bash
cd gamba
cargo build --release --target wasm32-unknown-unknown
```

### 3. Test Locally
```bash
# Run unit tests
cargo test

# Run specific orbital wand tests
cargo test test_orbital_wand
```

### 4. Deploy to Network
```bash
# Deploy to testnet first
alkane deploy --network testnet target/wasm32-unknown-unknown/release/gamba.wasm

# Deploy to mainnet (when ready)
alkane deploy --network mainnet target/wasm32-unknown-unknown/release/gamba.wasm
```

## Contract Interaction

### Initialize the Contract
```bash
alkane call <contract_id> --opcode 0  # Initialize
```

### Cast a Wand (Gamble)
```bash
# Send alkane ID to gamble
alkane call <contract_id> --opcode 42 --data <alkane_id_bytes>
```

### Query Contract State
```bash
# Get wand count
alkane call <contract_id> --opcode 2000

# Get latest wand SVG
alkane call <contract_id> --opcode 1000

# Get all wands
alkane call <contract_id> --opcode 2001
```

## Integration Examples

### JavaScript/TypeScript
```javascript
import { AlkaneClient } from 'alkane-sdk';

const client = new AlkaneClient('https://alkane-rpc.example.com');
const contractId = 'your-gamba-contract-id';

// Cast a wand
async function castWand(alkaneId) {
  try {
    const result = await client.call(contractId, {
      opcode: 42,
      data: alkaneId
    });
    console.log('🎉 Won an orbital wand!', result);
  } catch (error) {
    console.log('💸 Better luck next time!', error);
  }
}

// Get wand SVG
async function getWandSvg() {
  const svg = await client.call(contractId, { opcode: 1000 });
  return new TextDecoder().decode(svg.data);
}
```

### Python
```python
import alkane_sdk

client = alkane_sdk.Client('https://alkane-rpc.example.com')
contract_id = 'your-gamba-contract-id'

def cast_wand(alkane_id):
    try:
        result = client.call(contract_id, opcode=42, data=alkane_id)
        print("🎉 Won an orbital wand!", result)
        return result
    except Exception as e:
        print("💸 Better luck next time!", e)
        return None

def get_wand_svg():
    result = client.call(contract_id, opcode=1000)
    return result.data.decode('utf-8')
```

## Monitoring & Analytics

### Track Contract Events
```bash
# Monitor wand casting events
alkane logs <contract_id> --filter "cast_wand"

# Track win/loss statistics
alkane logs <contract_id> --filter "wand_minted"
```

### Query Statistics
```bash
# Total wands minted
alkane call <contract_id> --opcode 2000

# Get all wand metadata for analysis
alkane call <contract_id> --opcode 2001
```

## Security Checklist

- [ ] Contract deployed with correct parameters
- [ ] Anti-replay protection working (test duplicate transactions)
- [ ] XOR randomness functioning correctly
- [ ] SVG generation working for all power levels
- [ ] Storage limits appropriate for expected usage
- [ ] Access controls properly configured

## Troubleshooting

### Common Issues

**Build Errors**
```bash
# Clean and rebuild
cargo clean
cargo build --release --target wasm32-unknown-unknown
```

**Dependency Issues**
```bash
# Update dependencies
cargo update

# Check for conflicts
cargo tree
```

**Runtime Errors**
- Check alkane runtime version compatibility
- Verify WASM target is correct
- Ensure sufficient gas/fuel for operations

### Debug Mode
```bash
# Build with debug info
cargo build --target wasm32-unknown-unknown

# Run tests with output
cargo test -- --nocapture
```

## Performance Optimization

### Gas Usage
- Wand casting: ~50,000 gas
- SVG generation: ~30,000 gas
- Query operations: ~5,000 gas

### Storage Efficiency
- Each wand: 97 bytes
- Contract metadata: ~1KB
- Estimated capacity: 10,000+ wands per deployment

## Mainnet Deployment Checklist

- [ ] Thorough testing on testnet
- [ ] Security audit completed
- [ ] Gas costs optimized
- [ ] Documentation updated
- [ ] Community testing completed
- [ ] Backup and recovery plan ready
- [ ] Monitoring systems configured

---

**Ready to deploy? May the blockchain be with you!** 🎲✨