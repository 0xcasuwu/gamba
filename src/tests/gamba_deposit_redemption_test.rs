use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
use std::fmt::Write;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;

use crate::precompiled::factory_build;
use crate::precompiled::coupon_template_build;
use alkanes::precompiled::free_mint_build;

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

// Comprehensive gamba contract ecosystem setup
fn create_gamba_ecosystem_setup() -> Result<(AlkaneId, AlkaneId, OutPoint)> {
    clear();
    
    println!("🎰 GAMBA ECOSYSTEM: Contract Setup");
    println!("==================================");
    
    // PHASE 1: Deploy contract templates (including factory)
    println!("\n📦 PHASE 1: Deploying Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // free_mint template → deploys instance at block 4, tx 797 (opcode 0 for init)
            vec![3u128, 797u128, 101u128],
            // coupon_token template → deploys instance at block 4, tx 0x601 (opcode 0 for init)
            vec![3u128, 0x601, 0u128, 1u128, 1000u128, 50u128, 10u128, 60u128, 1u128, 1u128, 4u128, 0x701u128],
            // factory template → deploys instance at block 4, tx 0x701 (opcode 0 for init)
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    println!("✅ Contract templates deployed at block 0");
    
    // TRACE: Template deployment
    for (i, tx) in template_block.txdata.iter().enumerate() {
        println!("🔍 Template TX {} traces:", i);
        for vout in 0..3 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("   • vout {}: {:?}", vout, *trace_guard);
            }
        }
    }
    
    // PHASE 2: Initialize Free-Mint Contract
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
                                    6u128, 797u128, 0u128,  // Deploy to block 6, tx 797, opcode 0 (Initialize)
                                    1000000u128,            // token_units (initial supply)
                                    100000u128,             // value_per_mint  
                                    1000000000u128,         // cap (high cap for testing)
                                    0x54455354,             // name_part1 ("TEST")
                                    0x434f494e,             // name_part2 ("COIN")
                                    0x545354,               // symbol ("TST")
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
    index_block(&free_mint_block, 1)?;
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);
    
    // TRACE: Free-mint initialization
    println!("\n🔍 TRACE: Free-mint initialization");
    for vout in 0..3 {
        let trace_data = &view::trace(&OutPoint {
            txid: free_mint_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Free-mint init vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    // PHASE 3: Initialize Coupon Template
    println!("\n🎫 PHASE 3: Initializing Coupon Template");
    let coupon_template_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 0x601u128, 0u128,  // Deploy to block 6, tx 0x601, opcode 0 (Initialize)
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
    index_block(&coupon_template_block, 2)?;
    
    let coupon_template_id = AlkaneId { block: 6, tx: 0x601 };
    println!("✅ Coupon template initialized at {:?}", coupon_template_id);
    
    // PHASE 4: Factory is created by template deployment
    let factory_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Gamba factory created by template deployment at {:?}", factory_id);
    
    // Test what opcodes are available on the factory
    test_factory_opcodes(&factory_id)?;
    
    println!("\n🎉 GAMBA ECOSYSTEM SETUP COMPLETE!");
    println!("===================================");
    println!("✅ Free-mint contract: {:?}", free_mint_contract_id);
    println!("✅ Coupon template: {:?}", coupon_template_id);
    println!("✅ Gamba factory: {:?}", factory_id);
    println!("✅ Ready for deposit → coupon creation → redemption testing");
    
    // Return the stake token outpoint for later use
    let stake_token_outpoint = OutPoint {
        txid: free_mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    Ok((free_mint_contract_id, factory_id, stake_token_outpoint))
}

// Helper to create fresh stake tokens
fn create_stake_tokens(block_height: u32) -> Result<Block> {
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(block_height as u16),
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // MintTokens opcode
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
    index_block(&mint_block, block_height)?;
    
    println!("✅ Created fresh stake tokens at block {}", block_height);
    Ok(mint_block)
}

// Comprehensive deposit operation with trace analysis
fn perform_deposit_with_traces(
    mint_block: &Block, 
    factory_id: &AlkaneId, 
    stake_amount: u128, 
    user_name: &str, 
    block_height: u32
) -> Result<(Block, ProtoruneRuneId)> {
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Get available tokens
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("\n💰 {} DEPOSIT OPERATION", user_name.to_uppercase());
    println!("======================");
    println!("🔍 Available tokens: {}", available_tokens);
    println!("🎯 Stake amount: {}", stake_amount);
    
    if available_tokens < stake_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, stake_amount));
    }
    
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint,
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
                                    factory_id.block,
                                    factory_id.tx,
                                    1u128, // create_coupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: 2,
                                            tx: 1,
                                        },
                                        amount: available_tokens,
                                        output: 1,
                                    }
                                ],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&deposit_block, block_height)?;
    
    // COMPREHENSIVE DEPOSIT TRACE ANALYSIS
    println!("\n🔍 DEPOSIT TRACE ANALYSIS");
    println!("=========================");
    
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • {} deposit vout {} trace: {:?}", user_name, vout, *trace_guard);
        }
    }
    
    // Verify coupon token creation
    let coupon_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let coupon_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&coupon_outpoint)?)
    );
    
    println!("\n📊 COUPON TOKEN ANALYSIS");
    println!("==========================");
    for (id, amount) in coupon_sheet.balances().iter() {
        println!("   • Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Get the coupon token ID
    let coupon_token_info = coupon_sheet.cached.balances.iter()
        .find(|(id, _amount)| id.block != 2 || id.tx != 1) // Not the stake token
        .ok_or_else(|| anyhow::anyhow!("No coupon token found for {}", user_name))?;
    
    let coupon_token_id = ProtoruneRuneId {
        block: coupon_token_info.0.block,
        tx: coupon_token_info.0.tx,
    };
    
    println!("✅ {} deposit successful at block {}", user_name, block_height);
    println!("🎫 Coupon token: {:?}", coupon_token_id);
    
    Ok((deposit_block, coupon_token_id))
}

// Comprehensive redemption operation with trace analysis
fn perform_redemption_with_traces(
    deposit_block: &Block,
    coupon_token_id: &ProtoruneRuneId,
    factory_id: &AlkaneId,
    user_name: &str,
    block_height: u32
) -> Result<Block> {
    let coupon_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Get available coupon tokens
    let coupon_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&coupon_outpoint)?));
    let available_coupon_tokens = coupon_sheet.get(coupon_token_id);
    
    println!("\n🎰 {} REDEMPTION OPERATION", user_name.to_uppercase());
    println!("==========================");
    println!("🎫 Coupon tokens available: {}", available_coupon_tokens);
    
    if available_coupon_tokens == 0 {
        return Err(anyhow::anyhow!("No coupon tokens available for redemption"));
    }
    
    let redemption_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: coupon_outpoint,
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
                                    factory_id.block,
                                    factory_id.tx,
                                    60u128, // redeem_winning_coupon opcode
                                    coupon_token_id.block,
                                    coupon_token_id.tx,
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: coupon_token_id.clone(),
                                        amount: available_coupon_tokens,
                                        output: 1,
                                    }
                                ],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&redemption_block, block_height)?;
    
    // COMPREHENSIVE REDEMPTION TRACE ANALYSIS
    println!("\n🔍 REDEMPTION TRACE ANALYSIS");
    println!("============================");
    
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: redemption_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • {} redemption vout {} trace: {:?}", user_name, vout, *trace_guard);
        }
    }
    
    // Analyze redemption results
    let redemption_outpoint = OutPoint {
        txid: redemption_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let redemption_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&redemption_outpoint)?)
    );
    
    println!("\n💰 REDEMPTION RESULTS ANALYSIS");
    println!("==============================");
    let mut total_received = 0u128;
    for (id, amount) in redemption_sheet.balances().iter() {
        println!("   • Received Token ID: {:?}, Amount: {}", id, amount);
        total_received += amount;
    }
    
    println!("✅ {} redemption completed at block {}", user_name, block_height);
    println!("🏆 Total tokens received: {}", total_received);
    
    Ok(redemption_block)
}

// Simple test to check what contract is actually created at the factory address
fn test_factory_opcodes(factory_id: &AlkaneId) -> Result<()> {
    println!("\n🔍 TESTING FACTORY OPCODES");
    println!("==========================");
    println!("Testing factory at {:?}", factory_id);
    
    // Test opcode 0 (Initialize)
    let test_init_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_id.block,
                                    factory_id.tx,
                                    0u128, // Test opcode 0
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
    
    match index_block(&test_init_block, 10) {
        Ok(_) => println!("✅ Opcode 0 (Initialize) - RECOGNIZED"),
        Err(e) => println!("❌ Opcode 0 (Initialize) - ERROR: {}", e),
    }
    
    // Test opcode 1 (CreateCoupon)
    let test_create_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_id.block,
                                    factory_id.tx,
                                    1u128, // Test opcode 1
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
    
    match index_block(&test_create_block, 11) {
        Ok(_) => println!("✅ Opcode 1 (CreateCoupon) - RECOGNIZED"),
        Err(e) => println!("❌ Opcode 1 (CreateCoupon) - ERROR: {}", e),
    }
    
    Ok(())
}

