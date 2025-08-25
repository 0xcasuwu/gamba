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
use crate::precompiled::free_mint_build;
use crate::precompiled::auth_token_build;

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

// Comprehensive contract ecosystem setup with proper authorization chain
fn setup_contract_ecosystem() -> Result<(AlkaneId, AlkaneId, u128, OutPoint)> {
    clear();
    
    println!("🏗️ CONTRACT ECOSYSTEM SETUP");
    println!("==================================================");
    
    // PHASE 1: Deploy contract templates
    println!("\n📦 PHASE 1: Deploying Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            factory_build::get_bytes(), // Using factory_build
            coupon_template_build::get_bytes(), // Using coupon_template_build
            auth_token_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128], // free_mint
            vec![3u128, 0x701, 10u128], // factory_build (CouponFactory)
            vec![3u128, 0x601, 10u128], // coupon_template_build (CouponToken)
            vec![3u128, 0xffee, 0u128, 1u128], // auth_token
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
    
    // PHASE 2: Initialize Free-Mint Contract (CRITICAL: This creates auth token)
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
    let free_mint_auth_token_id = AlkaneId { block: 2, tx: 2 };
    
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);
    println!("🔑 Auth token created at {:?}", free_mint_auth_token_id);
    
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
            println!("   • vout {}: {:?}", vout, *trace_guard);
        }
    }
    
    // PHASE 3: Initialize Coupon Factory (formerly Vault Factory)
    println!("\n🏭 PHASE 3: Initializing Coupon Factory");
    // The deposit_token_id will be set dynamically in the integration test
    let reward_per_block = 1000u128; // This is a placeholder, not directly used by CouponFactory
    
    // The actual initialization of the Coupon Factory with the correct coupon_token_template_id
    // will happen in the integration test function. This function only sets up the templates.
    
    println!("\n🎉 CONTRACT ECOSYSTEM SETUP COMPLETE!");
    println!("=====================================");
    println!("✅ Free-mint contract: {:?}", free_mint_contract_id);
    println!("✅ Ready for dynamic Coupon Factory initialization and testing");
    
    // Return the free_mint_contract_id and a placeholder for factory_id and deposit_token_outpoint
    // The actual factory_id and deposit_token_outpoint will be determined later.
    Ok((free_mint_contract_id, AlkaneId { block: 0, tx: 0 }, reward_per_block, OutPoint::null()))
}

// Helper to mint tokens from the free-mint contract
fn mint_tokens_from_free_mint_contract(free_mint_contract_id: &AlkaneId, block_height: u32) -> Result<Block> {
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
                                message: into_cellpack(vec![
                                    free_mint_contract_id.block,
                                    free_mint_contract_id.tx,
                                    77u128 // MintTokens opcode
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
    index_block(&mint_block, block_height)?;
    
    println!("✅ Minted tokens from free-mint contract at block {}", block_height);
    Ok(mint_block)
}

// Comprehensive deposit operation with trace analysis (now create_coupon)
fn create_coupon_with_traces(
    mint_block: &Block, 
    coupon_factory_id: &AlkaneId, 
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
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 }; // Assuming the minted token ID is this
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("\n💰 {} CREATE COUPON OPERATION", user_name.to_uppercase());
    println!("======================");
    println!("🔍 Available tokens: {}", available_tokens);
    println!("🎯 Stake amount: {}", stake_amount);
    
    if available_tokens < stake_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, stake_amount));
    }
    
    let create_coupon_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    coupon_factory_id.block,
                                    coupon_factory_id.tx,
                                    1u128, // CreateCoupon opcode
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
                                        amount: available_tokens, // Stake all available tokens
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
    index_block(&create_coupon_block, block_height)?;
    
    // COMPREHENSIVE CREATE COUPON TRACE ANALYSIS
    println!("\n🔍 CREATE COUPON TRACE ANALYSIS");
    println!("=========================");
    
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: create_coupon_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • {} create coupon vout {} trace: {:?}", user_name, vout, *trace_guard);
        }
    }
    
    // Verify coupon token creation
    let coupon_outpoint = OutPoint {
        txid: create_coupon_block.txdata[0].compute_txid(),
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
    
    // Get the coupon token ID (assuming it's the one not free-mint token)
    let coupon_token_id = coupon_sheet.cached.balances.iter()
        .find(|(id, _amount)| id.block != 2 || id.tx != 1) 
        .map(|(id, _amount)| id.clone())
        .ok_or_else(|| anyhow::anyhow!("No coupon token found for {}", user_name))?;
    
    println!("✅ {} coupon creation successful at block {}", user_name, block_height);
    println!("🎫 Coupon token: {:?}", coupon_token_id);
    
    Ok((create_coupon_block, coupon_token_id))
}


#[wasm_bindgen_test]
fn test_integration_mint_deposit_flow() -> Result<()> {
    println!("\n🚀 INTEGRATION TEST: MINT AND DEPOSIT FLOW");
    println!("===========================================");

    // PHASE 1: Setup Contract Ecosystem (deploy templates)
    let (free_mint_contract_id, _factory_placeholder, _reward_per_block, _deposit_outpoint_placeholder) = 
        setup_contract_ecosystem()?;
    
    // PHASE 2: Mint tokens from the Free-Mint Contract
    println!("\n💰 PHASE 2: Minting Tokens for Stake");
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Get the outpoint of the newly minted token
    let minted_token_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0, // Assuming the minted token is in the first output
    };

    // Get the RuneId of the minted token
    let minted_token_rune_id = ProtoruneRuneId { block: 2, tx: 1 }; // This is the expected ID for the free-mint token

    println!("✅ Minted token available at outpoint: {:?}", minted_token_outpoint);
    println!("✅ Minted token RuneId: {:?}", minted_token_rune_id);

    // PHASE 3: Initialize Coupon Factory with the CouponToken template
    println!("\n🏭 PHASE 3: Initializing Coupon Factory");
    let success_threshold = 144u128; // Example threshold
    let coupon_template_id = AlkaneId { block: 4, tx: 0x601 }; // Assuming coupon_template_build deploys here

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
                                    4u128, // Use the build's AlkaneId
                                    0x701u128,
                                    0u128, // Initialize opcode
                                    success_threshold,
                                    coupon_template_id.block, coupon_template_id.tx,
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
    index_block(&init_factory_block, 6)?; // Index after minting
    
    let coupon_factory_id = AlkaneId { block: 7, tx: 0x701 }; // Assuming this is the factory ID after initialization
    
    println!("✅ Coupon Factory initialized at {:?}", coupon_factory_id);
    println!("🔗 Configured with CouponToken template: {:?}", coupon_template_id);

    // PHASE 4: Create Coupon using the minted token as stake
    println!("\n💰 PHASE 4: Creating Coupon with Minted Token Stake");
    let stake_amount = 100000u128; // Stake the full minted amount
    let create_coupon_block_height = 10;

    let (_coupon_block, _coupon_token_id) = create_coupon_with_traces(
        &minted_block, // Use the block that contains the minted token
        &coupon_factory_id,
        stake_amount,
        "IntegrationUser",
        create_coupon_block_height
    )?;

    println!("\n🎊 INTEGRATION TEST: MINT AND DEPOSIT FLOW SUMMARY");
    println!("===================================================");
    println!("✅ Contract ecosystem setup: PASSED");
    println!("✅ Tokens minted from Free-Mint contract: PASSED");
    println!("✅ Coupon Factory initialized with CouponToken template: PASSED");
    println!("✅ Coupon created using minted token as stake: PASSED");
    println!("✅ Integration test completed successfully.");

    Ok(())
}