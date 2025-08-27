# Deployment Pattern Analysis: Block 6 → 4 → 2 Pattern

## Critical Pattern Discovery

Both Boiler and Gamba use the same deployment pattern that is **NOT** the issue:

### Template Deployment Pattern
```
Template deployed at: block 4, tx 0x601 (or 0x385 for Boiler)
Factory calls template at: block 6, tx 0x601 
Template spawns new instance at: block 2, tx 1
```

### Pattern Flow
1. **Template Deployment**: `block 4, tx 0x601` (template lives here)
2. **Factory Call**: `block 6, tx 0x601` (factory calls template)
3. **Instance Spawn**: `block 2, tx 1` (new token instance created)

### Evidence from Working Boiler
```rust
// Template deployment
vec![3u128, 0x385u128] // deploys to block 4, tx 0x385

// Factory calls template
target: AlkaneId { block: 6, tx: POSITION_TOKEN_TEMPLATE_ID } // calls block 6, tx 0x385

// Instance spawns at block 2, tx 1
```

### Evidence from Gamba
```rust
// Template deployment  
vec![3u128, 0x601u128] // deploys to block 4, tx 0x601

// Factory calls template
target: AlkaneId { block: 6, tx: coupon_template_id } // calls block 6, tx 0x601

// Instance should spawn at block 2, tx 1
```

## Key Insight
The deployment pattern is **CORRECT** and **IDENTICAL** between Boiler and Gamba. The issue must be elsewhere in the token return mechanism.

## Next Investigation Areas
1. Template ID storage/retrieval in factory
2. Token return mechanism in coupon template
3. Response handling in factory
4. Trace analysis to see where tokens are actually going
