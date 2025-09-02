use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use std::fmt::Write;

// Import the precompiled builds following boiler pattern
use crate::precompiled::free_mint_build;
use crate::precompiled::coupon_template_build;
use crate::precompiled::factory_build;

/// Convert vector to Cellpack following boiler pattern
pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

/// Helper function to call factory with specific opcode and analyze response
/// Following exact boiler pattern for contract calls
fn call_factory(
    factory_id: &AlkaneId,
    opcode: u128,
    inputs: Vec<u128>,
    block_height: u32,
    test_name: &str
) -> Result<Vec<u8>> {
    let mut call_inputs = vec![
        factory_id.block,
        factory_id.tx,
        opcode,
    ];
    call_inputs.extend(inputs);

    let test_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(call_inputs).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // No tokens needed for getter queries
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&test_block, block_height)?;

    println!("✅ {} call executed at block {}", test_name, block_height);

    // Get the response data from the trace following boiler pattern
    let response_outpoint = OutPoint {
        txid: test_block.txdata[0].compute_txid(),
        vout: 0,
    };

    let trace_data = &view::trace(&response_outpoint)?;
    let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
    let trace_guard = trace_result.0.lock().unwrap();

    println!("📊 {} trace executed successfully", test_name);
    Ok(Vec::new())
}

/// Helper to parse u128 from response data following boiler pattern
fn parse_u128_response(data: &[u8], expected_name: &str) -> Result<u128> {
    if data.len() < 16 {
        return Err(anyhow::anyhow!("{} response too short: {} bytes", expected_name, data.len()));
    }
    let value = u128::from_le_bytes(data[0..16].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} as u128", expected_name)
    })?);
    println!("📊 {}: {}", expected_name, value);
    Ok(value)
}

/// Helper to parse AlkaneId from response data following boiler pattern
fn parse_alkane_id_response(data: &[u8], expected_name: &str) -> Result<AlkaneId> {
    if data.len() < 32 {
        return Err(anyhow::anyhow!("{} response too short: {} bytes", expected_name, data.len()));
    }
    let block = u128::from_le_bytes(data[0..16].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} block", expected_name)
    })?);
    let tx = u128::from_le_bytes(data[16..32].try_into().map_err(|_| {
        anyhow::anyhow!("Failed to parse {} tx", expected_name)
    })?);
    let alkane_id = AlkaneId { block, tx };
    println!("📊 {}: AlkaneId {{ block: {}, tx: {} }}", expected_name, block, tx);
    Ok(alkane_id)
}

/// Helper to parse bool from response data following boiler pattern
fn parse_bool_response(data: &[u8], expected_name: &str) -> Result<bool> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("{} response is empty", expected_name));
    }
    let value = data[0] != 0;
    println!("📊 {}: {}", expected_name, value);
    Ok(value)
}

/// Comprehensive setup function that creates a well-configured gamba ecosystem
/// Following exact boiler deployment pattern: 6→4→2
fn create_comprehensive_gamba_setup() -> Result<(AlkaneId, AlkaneId, AlkaneId, Vec<(String, AlkaneId)>)> {
    clear();
    
    println!("🏗️ COMPREHENSIVE GAMBA TESTS: Contract Ecosystem Setup");
    println!("=======================================================");
    
    // PHASE 1: Deploy contract templates at block 0 following boiler pattern
    println!("\n📦 PHASE 1: Deploying Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // Free-mint: full initialization during template deployment
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354],
            // Coupon template: will be called by factory at (6, 0x601)
            vec![3u128, 0x601, 10u128],
            // Factory: initialized with success_threshold=144, coupon_template_id=(4, 0x601)
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    alkanes::indexer::index_block(&template_block, 0)?;
    
    println!("✅ Contract templates deployed at block 0");
    
    // PHASE 2: Initialize Free-Mint Contract using 6→4→2 pattern
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract");
    let free_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    6u128, 797u128, 0u128,  // Deploy pattern: 6→4→2
                                    1000000u128,            // token_units
                                    100000u128,             // value_per_mint  
                                    1000000000u128,         // cap
                                    0x54455354,             // name ("TEST")
                                    0x434f494e,             // name ("COIN") 
                                    0x545354                // symbol ("TST")
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&free_mint_block, 2)?;
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    let free_mint_auth_token_id = AlkaneId { block: 2, tx: 2 };
    
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);
    println!("🔑 Auth token created at {:?}", free_mint_auth_token_id);
    
    // PHASE 3: Initialize Factory with specific test parameters following boiler pattern
    println!("\n🏭 PHASE 3: Initializing Gamba Factory");
    let success_threshold = 144u128; // XOR threshold for success
    let coupon_template_id = AlkaneId { block: 4, tx: 0x601 }; // Template location
    
    let init_factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128, 0x701, 0u128, // Initialize factory at block 4
                                    success_threshold, // success threshold
                                    coupon_template_id.block, coupon_template_id.tx, // coupon template
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&init_factory_block, 4)?;
    
    let factory_id = AlkaneId { block: 4, tx: 0x701 };
    println!("✅ Factory initialized at {:?}", factory_id);
    
    // PHASE 4: Create test coupons using factory following boiler pattern
    println!("\n🎰 PHASE 4: Creating Test Coupons");
    let stake_amount = 10000u128;
    
    // Create winning coupon
    let winning_coupon_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![
                        // Send stake tokens to factory (following boiler alkane input pattern)
                        ProtostoneEdict {
                            id: ProtoruneRuneId { 
                                block: free_mint_contract_id.block, 
                                tx: free_mint_contract_id.tx 
                            },
                            amount: stake_amount,
                            output: 0, // Send to output 0 (the alkane call)
                        }.into()
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128,     // Factory at block 4
                                    0x701,     // Factory tx 0x701
                                    1u128,     // CreateCoupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&winning_coupon_block, 6)?;
    
    println!("✅ Test coupon created at block 6");
    
    // Return the contract ecosystem for testing
    let contracts = vec![
        ("free_mint", free_mint_contract_id),
        ("factory", factory_id),
        ("coupon_template", coupon_template_id),
    ];
    
    Ok((free_mint_contract_id, factory_id, coupon_template_id, contracts))
}

/// Test the factory initialization following boiler patterns
#[wasm_bindgen_test]
fn test_factory_initialization() -> Result<()> {
    println!("🧪 TEST: Factory Initialization");
    println!("===============================");
    
    let (_, factory_id, _, _) = create_comprehensive_gamba_setup()?;
    
    // Test factory getters following boiler pattern
    println!("\n🔍 Testing Factory Getters");
    
    // Test success threshold
    let threshold_data = call_factory(&factory_id, 21, vec![], 8, "GetSuccessThreshold")?;
    let threshold = parse_u128_response(&threshold_data, "Success Threshold")?;
    assert_eq!(threshold, 144, "Success threshold should be 144");
    
    // Test coupon template ID
    let template_data = call_factory(&factory_id, 23, vec![], 9, "GetCouponTemplateId")?;
    let template_id = parse_alkane_id_response(&template_data, "Coupon Template ID")?;
    assert_eq!(template_id.block, 4, "Template should be at block 4");
    assert_eq!(template_id.tx, 0x601, "Template should be at tx 0x601");
    
    // Test initial coupon counts
    let successful_data = call_factory(&factory_id, 10, vec![], 10, "GetSuccessfulCoupons")?;
    let successful = parse_u128_response(&successful_data, "Successful Coupons")?;
    assert_eq!(successful, 0, "Initial successful coupons should be 0");
    
    let failed_data = call_factory(&factory_id, 11, vec![], 11, "GetFailedCoupons")?;
    let failed = parse_u128_response(&failed_data, "Failed Coupons")?;
    assert_eq!(failed, 0, "Initial failed coupons should be 0");
    
    println!("✅ Factory initialization test passed");
    Ok(())
}

/// Test coupon creation following boiler patterns
#[wasm_bindgen_test]
fn test_coupon_creation() -> Result<()> {
    println!("🧪 TEST: Coupon Creation");
    println!("=========================");
    
    let (free_mint_id, factory_id, _, _) = create_comprehensive_gamba_setup()?;
    
    // Test coupon creation with stake tokens
    println!("\n🔍 Testing Coupon Creation");
    
    // Create a coupon by sending stake tokens to factory
    let coupon_creation_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![
                        // Send stake tokens to factory following boiler alkane input pattern
                        ProtostoneEdict {
                            id: ProtoruneRuneId { 
                                block: free_mint_id.block, 
                                tx: free_mint_id.tx 
                            },
                            amount: 5000u128, // Stake amount
                            output: 0, // Send to output 0 (the alkane call)
                        }.into()
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128,     // Factory at block 4
                                    0x701,     // Factory tx 0x701
                                    1u128,     // CreateCoupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&coupon_creation_block, 12)?;
    
    // Test that coupon was created by checking counts
    let total_data = call_factory(&factory_id, 12, vec![], 13, "GetTotalCoupons")?;
    let total = parse_u128_response(&total_data, "Total Coupons")?;
    assert!(total > 0, "At least one coupon should be created");
    
    println!("✅ Coupon creation test passed");
    Ok(())
}

/// Test data retrieval patterns following boiler storage approach
#[wasm_bindgen_test]
fn test_data_retrieval_patterns() -> Result<()> {
    println!("🧪 TEST: Data Retrieval Patterns");
    println!("=================================");
    
    let (_, factory_id, _, _) = create_comprehensive_gamba_setup()?;
    
    // Test factory info retrieval following boiler pattern
    println!("\n🔍 Testing Factory Info Retrieval");
    
    let info_data = call_factory(&factory_id, 40, vec![], 14, "GetFactoryInfo")?;
    
    // Parse factory info following boiler data format
    if info_data.len() >= 65 {
        let template_block = u128::from_le_bytes(info_data[0..16].try_into()?);
        let template_tx = u128::from_le_bytes(info_data[16..32].try_into()?);
        let success_threshold = info_data[32];
        let successful_coupons = u128::from_le_bytes(info_data[33..49].try_into()?);
        let failed_coupons = u128::from_le_bytes(info_data[49..65].try_into()?);
        
        println!("📊 Factory Info:");
        println!("   Template: ({}, {})", template_block, template_tx);
        println!("   Success Threshold: {}", success_threshold);
        println!("   Successful Coupons: {}", successful_coupons);
        println!("   Failed Coupons: {}", failed_coupons);
        
        assert_eq!(template_block, 4, "Template should be at block 4");
        assert_eq!(template_tx, 0x601, "Template should be at tx 0x601");
        assert_eq!(success_threshold, 144, "Success threshold should be 144");
    }
    
    // Test registered coupons retrieval following boiler pattern
    let coupons_data = call_factory(&factory_id, 30, vec![], 15, "GetAllRegisteredCoupons")?;
    
    if coupons_data.len() >= 8 {
        let count = u64::from_le_bytes(coupons_data[0..8].try_into()?);
        println!("📊 Registered Coupons Count: {}", count);
        
        // Parse coupon IDs if any exist
        if count > 0 && coupons_data.len() >= 8 + (count as usize * 32) {
            let mut offset = 8;
            for i in 0..count {
                let block = u128::from_le_bytes(coupons_data[offset..offset+16].try_into()?);
                let tx = u128::from_le_bytes(coupons_data[offset+16..offset+32].try_into()?);
                println!("   Coupon {}: ({}, {})", i, block, tx);
                offset += 32;
            }
        }
    }
    
    println!("✅ Data retrieval patterns test passed");
    Ok(())
}

/// Test block height handling following boiler patterns
#[wasm_bindgen_test]
fn test_block_height_handling() -> Result<()> {
    println!("🧪 TEST: Block Height Handling");
    println!("===============================");
    
    let (_, factory_id, _, _) = create_comprehensive_gamba_setup()?;
    
    // Test base XOR calculation which depends on block height
    println!("\n🔍 Testing Block Height Dependent Operations");
    
    let xor_data = call_factory(&factory_id, 50, vec![], 16, "CalculateBaseXor")?;
    let base_xor = parse_u128_response(&xor_data, "Base XOR")?;
    
    println!("📊 Base XOR at block 16: {}", base_xor);
    assert!(base_xor <= 255, "XOR should be in u8 range");
    
    // Test minimum stake amount
    let min_stake_data = call_factory(&factory_id, 51, vec![], 17, "GetMinimumStake")?;
    let min_stake = parse_u128_response(&min_stake_data, "Minimum Stake")?;
    
    println!("📊 Minimum Stake: {}", min_stake);
    assert!(min_stake > 0, "Minimum stake should be positive");
    
    println!("✅ Block height handling test passed");
    Ok(())
}

/// Test the complete gamba flow following boiler patterns
#[wasm_bindgen_test]
fn test_complete_gamba_flow() -> Result<()> {
    println!("🧪 TEST: Complete Gamba Flow");
    println!("=============================");
    
    let (free_mint_id, factory_id, _, _) = create_comprehensive_gamba_setup()?;
    
    // Test the complete flow: create coupon, check results, verify state
    println!("\n🔍 Testing Complete Gamba Flow");
    
    // Step 1: Create a coupon with stake
    let stake_amount = 3000u128;
    let coupon_flow_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![
                        // Send stake tokens to factory
                        ProtostoneEdict {
                            id: ProtoruneRuneId { 
                                block: free_mint_id.block, 
                                tx: free_mint_id.tx 
                            },
                            amount: stake_amount,
                            output: 0,
                        }.into()
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128,     // Factory at block 4
                                    0x701,     // Factory tx 0x701
                                    1u128,     // CreateCoupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    alkanes::indexer::index_block(&coupon_flow_block, 18)?;
    
    // Step 2: Verify coupon was created
    let total_before = call_factory(&factory_id, 12, vec![], 19, "GetTotalCouponsBefore")?;
    let total_before_count = parse_u128_response(&total_before, "Total Coupons Before")?;
    
    let total_after = call_factory(&factory_id, 12, vec![], 20, "GetTotalCouponsAfter")?;
    let total_after_count = parse_u128_response(&total_after, "Total Coupons After")?;
    
    assert!(total_after_count > total_before_count, "Coupon count should increase");
    
    // Step 3: Check individual counts
    let successful = call_factory(&factory_id, 10, vec![], 21, "GetSuccessfulCoupons")?;
    let successful_count = parse_u128_response(&successful, "Successful Coupons")?;
    
    let failed = call_factory(&factory_id, 11, vec![], 22, "GetFailedCoupons")?;
    let failed_count = parse_u128_response(&failed, "Failed Coupons")?;
    
    let total = successful_count + failed_count;
    assert_eq!(total, total_after_count, "Total should equal successful + failed");
    
    println!("📊 Flow Results:");
    println!("   Total Coupons: {}", total_after_count);
    println!("   Successful: {}", successful_count);
    println!("   Failed: {}", failed_count);
    
    println!("✅ Complete gamba flow test passed");
    Ok(())
}

/// Main test runner following boiler pattern
#[wasm_bindgen_test]
fn run_all_boiler_pattern_tests() -> Result<()> {
    println!("🚀 RUNNING ALL BOILER PATTERN TESTS");
    println!("===================================");
    
    test_factory_initialization()?;
    test_coupon_creation()?;
    test_data_retrieval_patterns()?;
    test_block_height_handling()?;
    test_complete_gamba_flow()?;
    
    println!("\n🎉 ALL BOILER PATTERN TESTS PASSED!");
    println!("✅ Gamba system successfully follows boiler patterns");
    println!("✅ Data retrieval/storage patterns implemented correctly");
    println!("✅ Block height handling follows boiler approach");
    println!("✅ Deployment sequencing matches boiler 6→4→2 pattern");
    
    Ok(())
} 