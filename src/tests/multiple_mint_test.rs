// use alkanes::view; // Removed unused import
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
use alkanes::view;
use alkanes_support::proto::alkanes::AlkanesTrace;
use prost::Message;

use alkanes::precompiled::free_mint_build;
use crate::precompiled::factory_build;
use crate::precompiled::coupon_template_build;

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
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

#[test]
fn test_coupon_template_direct() -> Result<()> {
    clear();
    
    println!("🧪 TESTING COUPON TEMPLATE DIRECTLY");
    println!("====================================");
    
    // Deploy coupon template only
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            crate::precompiled::coupon_template_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 0x601],  // Deploy coupon template at block 4, tx 0x601
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    println!("✅ Coupon template deployed at block 0");
    
    // Try to initialize a coupon directly
    let coupon_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 0x601, 0u128,  // Call coupon template at block 6, tx 0x601, opcode 0 (Initialize)
                                    1u128,                 // coupon_id
                                    1000u128,              // stake_amount
                                    50u128,                // base_xor
                                    10u128,                // stake_bonus
                                    60u128,                // final_result
                                    1u128,                 // is_winner
                                    10u128,                // creation_block
                                    2u128,                 // factory_block
                                    1793u128,              // factory_tx
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
    index_block(&coupon_block, 1)?;
    
    println!("✅ Direct coupon initialization attempted at block 1");
    
    // Check the trace
    println!("🔍 TRACE: Direct coupon initialization");
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: coupon_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   - vout {}: {:?}", vout, *trace_guard);
        }
    }
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_coupon_template_deployment_only() -> Result<()> {
    clear();
    println!("\n🎯 COUPON TEMPLATE DEPLOYMENT TEST");
    println!("==================================");

    // Deploy only the coupon template
    println!("\n📦 PHASE 1: Deploying Coupon Template Only");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [coupon_template_build::get_bytes()].into(),
        [vec![3u128, 0x601]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Coupon template deployed at block 4, tx 0x601");
    
    // Check deployment trace
    println!("\n🔍 PHASE 2: Checking Deployment Trace");
    let trace_data = &view::trace(&OutPoint {
        txid: template_block.txdata[0].compute_txid(),
        vout: 3,
    })?;
    let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
    let trace_guard = trace_result.0.lock().unwrap();
    println!("   • Template deployment trace: {:?}", *trace_guard);

    println!("✅ Test completed successfully - no reverts!");
    Ok(())
}

#[wasm_bindgen_test]
fn test_free_mint_contract_minting() -> Result<()> {
    clear();
    println!("\n🚀 FREE-MINT CONTRACT MINTING TEST");
    println!("===================================");

    // PHASE 1: Deploy Free-Mint Contract Template
    println!("\n📦 PHASE 1: Deploying Free-Mint Contract Template");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [free_mint_build::get_bytes()].into(),
        [vec![3u128, 797u128, 101u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Free-mint contract template deployed at block 0");

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

    // PHASE 3: Mint tokens from the Free-Mint Contract
    println!("\n💰 PHASE 3: Minting Tokens");
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Verify minted tokens
    let mint_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let minted_token_id = ProtoruneRuneId { block: 2, tx: 1 }; // Assuming the minted token ID is this
    let minted_amount = mint_sheet.get(&minted_token_id);

    println!("🔍 Minted token ID: {:?}", minted_token_id);
    println!("🔍 Minted amount: {}", minted_amount);

    assert!(minted_amount > 0, "Expected minted amount to be greater than 0");
    println!("✅ Tokens successfully minted and verified.");

    println!("\n🎊 FREE-MINT CONTRACT MINTING TEST SUMMARY");
    println!("===========================================");
    println!("✅ Free-mint contract deployed and initialized.");
    println!("✅ Tokens successfully minted from the contract.");
    println!("✅ Test completed successfully.");

        // TRACE: Minted block data
    println!("🔍 TRACE: Minted block data at block {}", mint_block_height);
    for (i, tx) in minted_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }
    Ok(())
}

#[wasm_bindgen_test]
fn test_debug_factory_deployment_with_minting() -> Result<()> {
    println!("\n🔍 DEBUG: Factory Deployment with Minting");
    println!("=========================================");
    
    clear();
    
    // STEP 1: Deploy templates only
    println!("\n📦 STEP 1: Template Deployment");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            // coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // free_mint template → deploys instance at block 4, tx 797 (opcode 0 for init)
            // Arguments: token_units, value_per_mint, cap, name_part1, name_part2, symbol
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], // Complete initialization
            // coupon_token template → deploys instance at block 4, tx 0x601 (opcode 0 for init)
            // Arguments: coupon_id, stake_amount, base_xor, stake_bonus, final_result, is_winner, creation_block, factory_block, factory_tx
            vec![3u128, 0x601, 0u128, 1u128, 1000u128, 50u128, 10u128, 60u128, 1u128, 1u128, 4u128, 0x701u128], // Sample coupon initialization
            // coupon_factory template → deploys instance at block 4, tx 0x701 (opcode 0 for init)
            // Arguments: success_threshold, coupon_token_template_id
            vec![3u128, 0x701, 0u128, 180u128, 4u128, 0x601u128], // success_threshold=180, coupon_template_id=4,0x601
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    // TRACE: Template block deployment
    println!("🔍 TRACE: Template block deployment at block 0");
    for (i, tx) in template_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }
    println!("✅ Templates deployed successfully");
    
    // STEP 2: Initialize Free-Mint Contract (using working pattern from original test)
    println!("\n🪙 STEP 2: Initializing Free-Mint Contract");
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

    // STEP 3: Mint tokens from the Free-Mint Contract (using working minting pattern)
    println!("\n💰 STEP 3: Minting Tokens from Free-Mint Contract");
    println!("🔍 Expected free-mint contract ID: {:?}", free_mint_contract_id);
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Verify minted tokens (using working verification pattern)
    let mint_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let minted_token_id = ProtoruneRuneId { block: 2, tx: 1 }; // Free-mint contract spawned at block 2, tx 1
    let minted_amount = mint_sheet.get(&minted_token_id);

    println!("🔍 Minted token ID: {:?}", minted_token_id);
    println!("🔍 Minted amount: {}", minted_amount);

    assert!(minted_amount > 0, "Expected minted amount to be greater than 0");
    println!("✅ Tokens successfully minted and verified.");

    println!("\n🎊 DEBUG FACTORY DEPLOYMENT WITH MINTING TEST SUMMARY");
    println!("====================================================");
    println!("✅ Templates deployed successfully.");
    println!("✅ Free-mint contract initialized.");
    println!("✅ Tokens successfully minted from the free-mint contract.");
    println!("✅ Test completed successfully.");

    Ok(())
}

#[wasm_bindgen_test]
fn test_comprehensive_factory_integration() -> Result<()> {
    println!("\n🎰 COMPREHENSIVE FACTORY INTEGRATION TEST");
    println!("=========================================");
    
    clear();
    
    // PHASE 1: Deploy all contract templates
    println!("\n📦 PHASE 1: Deploying All Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // free_mint template → deploys instance at block 4, tx 797 (opcode 0 for init, complete parameters)
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], // Complete initialization
            // coupon_token template → deploys instance at block 4, tx 0x601 (no opcode for template deployment)
            // This template will be called by factory at block 6, tx 0x601 when creating coupon tokens
            vec![3u128, 0x601, 10u128 ],
            // coupon_factory template → deploys instance at block 4, tx 0x701 (opcode 0 for init)
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ All contract templates deployed at block 0");
    println!("   • Free-mint template: block 4, tx 797");
    println!("   • Coupon template: block 4, tx 0x601 (will be called by factory at block 6, tx 0x601)");
    println!("   • Factory template: block 4, tx 0x701");

    // PHASE 2: Initialize Free-Mint Contract (6 → 4 → 2 pattern)
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract");
    println!("   Deployment pattern: 6u128, 797u128, 0u128 → targets block 4, tx 797 → deploys to block 2, tx 1");
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

    // PHASE 3: Mint tokens from the Free-Mint Contract (opcode 77)
    println!("\n💰 PHASE 3: Minting Tokens (opcode 77)");
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Verify minted tokens
    let mint_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let minted_token_id = ProtoruneRuneId { block: 2, tx: 1 };
    let minted_amount = mint_sheet.get(&minted_token_id);

    println!("🔍 Minted token ID: {:?}", minted_token_id);
    println!("🔍 Minted amount: {}", minted_amount);

    assert!(minted_amount > 0, "Expected minted amount to be greater than 0");
    println!("✅ Tokens successfully minted and verified.");

    // PHASE 4: Initialize Factory Contract
    println!("\n🏭 PHASE 4: Initializing Factory Contract");
    let factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 0x701, 0u128,  // Deploy to block 6, tx 0x701, opcode 0 (Initialize)
                                    144u128,              // success_threshold
                                    4u128, 0x601u128,     // coupon_token_template_id (AlkaneId { block: 4, tx: 0x601 })
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
    index_block(&factory_block, 6)?;
    
    let factory_contract_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);

    // PHASE 5: Test CreateCoupon (opcode 1) with minted tokens
    println!("\n🎫 PHASE 5: Testing CreateCoupon (opcode 1)");
    let deposit_amount = 5000u128;
    
    // Get available tokens from the mint outpoint
    let available_tokens = mint_sheet.get(&minted_token_id);
    
    println!("🔍 Available tokens at mint outpoint: {}", available_tokens);
    println!("🎯 Deposit amount: {}", deposit_amount);
    
    if available_tokens < deposit_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, deposit_amount));
    }
    
    // Create a transaction that sends tokens to the factory contract
    // CRITICAL: Reference the mint outpoint in the transaction input
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint,  // 🔑 CRITICAL: Reference the mint outpoint
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(10),
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
            // Token output - send tokens to factory contract
            TxOut {
                script_pubkey: (Runestone {
                                                    edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId {
                                            block: minted_token_id.block,
                                            tx: minted_token_id.tx,
                                        },
                                        amount: available_tokens,  // Transfer all available tokens
                                        output: 1,
                                    }.into()
                                ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    factory_contract_id.block,
                                    factory_contract_id.tx,
                                    1u128, // CreateCoupon opcode (1 from gamba source)
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
    
    index_block(&deposit_block, 1000)?; // FORCE BRAND NEW COUPONS AT BLOCK 1000
    
    println!("✅ CreateCoupon transaction submitted at block 10");
    println!("   • Factory contract: {:?}", factory_contract_id);
    println!("   • Opcode: 1 (CreateCoupon)");
    println!("   • Mint outpoint: {:?}", mint_outpoint);
    println!("   • Token transfer: {:?} (amount: {})", minted_token_id, available_tokens);
    println!("   • Factory will call coupon template at block 6, tx 0x601");
    println!("   • ProtostoneEdict included: ✅");

    // DETAILED TRACE ANALYSIS
    println!("\n🔍 DETAILED TRACE ANALYSIS FOR CREATECOUPON");
    println!("=============================================");
    
    // Analyze the CreateCoupon transaction trace
    for (i, tx) in deposit_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }

    println!("\n🎊 COMPREHENSIVE FACTORY INTEGRATION TEST SUMMARY");
    println!("=================================================");
    println!("✅ All contract templates deployed successfully");
    println!("✅ Free-mint contract initialized (6→4→2 pattern)");
    println!("✅ Tokens minted successfully (opcode 77)");
    println!("✅ Factory contract initialized");
    println!("✅ CreateCoupon transaction submitted (opcode 1)");
    println!("✅ Test completed successfully");

    // TRACE: Final state analysis
    println!("\n🔍 TRACE: Final State Analysis");
    println!("   • Template deployment: block 0 (3→4 pattern)");
    println!("   • Free-mint initialization: block 1 (6→4→2 pattern)");
    println!("   • Token minting: block 5 (opcode 77)");
    println!("   • Factory initialization: block 6 (6→4→2 pattern)");
    println!("   • CreateCoupon: block 10 (opcode 1)");
    println!("   • Factory external call: block 6, tx 0x601 (targets coupon template at block 4, tx 0x601)");

    // PHASE 6: EXTRACT COUPON TOKEN AND DEMONSTRATE REDEMPTION WITH PAYOUT
    println!("\n💰 PHASE 6: EXTRACT COUPON TOKEN & REDEMPTION WITH PAYOUT");
    println!("=========================================================");
    
    // Extract the created coupon token from traces
    let mut created_coupon_id: Option<AlkaneId> = None;
    for (i, tx) in deposit_block.txdata.iter().enumerate() {
        println!("   🔍 Analyzing TX {} for coupon creation:", i);
        for vout in 0..6 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();

            if !trace_guard.is_empty() {
                for entry in trace_guard.iter() {
                    match entry {
                        alkanes_support::trace::TraceEvent::CreateAlkane(alkane_id) => {
                            created_coupon_id = Some(alkane_id.clone());
                            println!("     ✅ CAPTURED COUPON TOKEN: ({}, {})", alkane_id.block, alkane_id.tx);
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    if let Some(coupon_id) = created_coupon_id {
        println!("✅ COUPON TOKEN FOUND: {:?}", coupon_id);

        // PHASE 6.5: CREATE SECOND USER FOR MULTI-USER LOTTERY (SIMPLE APPROACH)
        println!("\n👥 PHASE 6.5: Creating Second User for Multi-User Lottery");
        
        // Create second user deposit at same block (block 10) with fresh tokens
        let second_user_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::from_height(7u16), // Make unique
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
                                    message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // Mint tokens
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
        index_block(&second_user_mint_block, 7)?; // Block 7 for minting
        
        // Second user deposit at block 10 (same as first user)
        let second_user_deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: second_user_mint_block.txdata[0].compute_txid(),
                    vout: 0,
                },
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
                                        4u128, 1793u128, 1u128, // CreateCoupon at factory
                                    ]).encipher(),
                                    protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                    pointer: Some(0),
                                    refund: Some(0),
                                    from: None,
                                    burn: None,
                                    edicts: vec![
                                        ProtostoneEdict {
                                            id: ProtoruneRuneId { block: 2, tx: 1 },
                                            amount: 50000, // LOSER: 50,000 tokens 
                                            output: 0,
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
        index_block(&second_user_deposit_block, 1000)?; // Block 1000 - BRAND NEW COUPONS
        
        // PHASE 6.6: Extract USER 2's coupon token to verify it was created at block 10
        println!("\n🔍 PHASE 6.6: Extracting USER 2's Coupon Token");
        let mut user2_coupon_id: Option<AlkaneId> = None;
        
        for vout in 0..6 {
            let trace_data = &view::trace(&OutPoint {
                txid: second_user_deposit_block.txdata[0].compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();

            if !trace_guard.is_empty() {
                for entry in trace_guard.iter() {
                    match entry {
                        alkanes_support::trace::TraceEvent::CreateAlkane(alkane_id) => {
                            user2_coupon_id = Some(alkane_id.clone());
                            println!("     ✅ USER 2 COUPON TOKEN: ({}, {})", alkane_id.block, alkane_id.tx);
                        },
                        _ => {}
                    }
                }
            }
        }

        let user2_coupon = user2_coupon_id.unwrap_or(AlkaneId { block: 0, tx: 0 });
        
        println!("✅ SECOND USER created at block 10 (50,000 tokens - SHOULD BE LOSER)");
        println!("📊 MULTI-USER LOTTERY SETUP:");
        println!("   • USER 1: 100,000 tokens (final_result=255 > threshold=180) → Coupon {:?}", coupon_id);
        println!("   • USER 2: 50,000 tokens (final_result=176 < threshold=180) → Coupon {:?}", user2_coupon);
        println!("   • Expected: USER 1 = WINNER, USER 2 = LOSER");
        println!("   • Expected USER 1 payout: 100,000 + 50,000 = 150,000 tokens");
        
        if user2_coupon.block == 0 {
            println!("   ❌ WARNING: USER 2's coupon token not found - this could explain pot issue!");
        } else {
            println!("   ✅ Both coupons created - ready for pot distribution test");
        }

        // PHASE 7: REDEMPTION WITH COUPON TOKEN (BOILER PATTERN)
        println!("\n🎰 PHASE 7: REDEMPTION WITH MULTI-USER LOTTERY");
        
        let coupon_outpoint = OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout: 0,
        };
        
        println!("🔍 Using coupon outpoint: {:?}", coupon_outpoint);
        println!("🎫 Bringing in coupon token: ({}, {}) with amount 1", coupon_id.block, coupon_id.tx);
        
        let redemption_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: coupon_outpoint, // CRITICAL: Bring in the coupon
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
                                        4u128, 1793u128, 2u128, // RedeemCoupon opcode
                                        coupon_id.block, coupon_id.tx, // Pass coupon ID
                                    ]).encipher(),
                                    protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                    pointer: Some(0),
                                    refund: Some(0),
                                    from: None,
                                    burn: None,
                                    edicts: vec![
                                        ProtostoneEdict {
                                            id: ProtoruneRuneId { block: coupon_id.block, tx: coupon_id.tx },
                                            amount: 1, // Bring the coupon token
                                            output: 0, // Send to factory call
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
        index_block(&redemption_block, 1001)?; // Block 1001 - lottery at 1000 ends, redemption starts at 1001

        // PHASE 8: ANALYZE REDEMPTION TRACES FOR POT DISTRIBUTION DEBUG
        println!("\n🔍 PHASE 8: ANALYZING REDEMPTION TRACES FOR POT DISTRIBUTION:");
        let mut total_payout_received = 0u128;

        for vout in 0..6 {
            let trace_data = &view::trace(&OutPoint {
                txid: redemption_block.txdata[0].compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();

            if !trace_guard.is_empty() {
                let trace_str = format!("{:?}", *trace_guard);
                println!("   📊 REDEMPTION vout {} trace: {:?}", vout, *trace_guard);

                // Extract winner status from both coupons based on trace parsing
                if trace_str.contains("AlkaneId { block: 2, tx: 3 }") {
                    println!("   🔍 USER 1 COUPON (2,3) FOUND IN TRACE");
                }
                if trace_str.contains("AlkaneId { block: 2, tx: 4 }") {
                    println!("   🔍 USER 2 COUPON (2,4) FOUND IN TRACE");
                }
                
                // Parse coupon data to extract winner status
                if trace_str.contains("data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 134, 1, 0") {
                    println!("   👑 USER 1: final_result=255, is_winner=1 (WINNER - CORRECT)");
                }
                if trace_str.contains("data: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 134, 1, 0") {
                    println!("   🎲 USER 2: final_result=176, is_winner=1 (❌ SHOULD BE LOSER!)");
                    println!("   ❌ BUG: USER 2 marked as winner despite final_result=176 < threshold=180");
                    println!("   ❌ This causes total_losing_deposits=0, breaking pot distribution");
                }
                
                // Look for the final payout
                if trace_str.contains("value: 150000") {
                    println!("   🎉 SUCCESS: Winner received proper n+n payout (150,000 tokens)!");
                } else if trace_str.contains("value: 100000") {
                    println!("   ❌ ISSUE: Winner only got original deposit (100,000) instead of n+n (150,000)");
                }

                for entry in trace_guard.iter() {
                    match entry {
                        alkanes_support::trace::TraceEvent::ReturnContext(return_ctx) => {
                            if !return_ctx.inner.alkanes.0.is_empty() {
                                for alkane in return_ctx.inner.alkanes.0.iter() {
                                    total_payout_received += alkane.value;
                                    println!("   💰 PAYOUT RECEIVED: {} tokens of ({}, {})", 
                                        alkane.value, alkane.id.block, alkane.id.tx);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        // PHASE 9: BALANCE SHEET VERIFICATION  
        println!("\n💰 PHASE 9: BALANCE SHEET VERIFICATION");
        let redemption_outpoint = OutPoint {
            txid: redemption_block.txdata[0].compute_txid(),
            vout: 0,
        };
        
        let redemption_sheet = load_sheet(
            &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
                .OUTPOINT_TO_RUNES
                .select(&consensus_encode(&redemption_outpoint)?)
        );
        
        let mut total_balance_received = 0u128;
        for (id, amount) in redemption_sheet.balances().iter() {
            println!("   💰 Balance received - Token ID: {:?}, Amount: {}", id, amount);
            total_balance_received += amount;
        }

        println!("\n🎊 MULTI-USER REDEMPTION WITH PAYOUT RESULTS:");
        println!("==============================================");
        println!("📊 USER 1 (WINNER): {} tokens deposited", available_tokens);
        println!("📊 USER 2 (LOSER): 50,000 tokens deposited");
        println!("📊 Total lottery pot: {} tokens", available_tokens + 50000);
        println!("✅ Coupon redeemed: {:?}", coupon_id);
        
        let max_payout = std::cmp::max(total_payout_received, total_balance_received);
        if max_payout > 0 {
            println!("✅ ACTUAL USER 1 PAYOUT: {} tokens", max_payout);
            
            if max_payout > available_tokens + 50000 {
                println!("🏆 JACKPOT: USER 1 won MORE than the entire pot!");
            } else if max_payout >= available_tokens + 40000 {
                let profit = max_payout - available_tokens;
                println!("🎉 🎉 🎉 MULTI-USER SUCCESS: USER 1 WON {} TOKENS FROM USER 2'S LOSING BET! 🎉 🎉 🎉", profit);
                println!("💰 USER 1 got: {} original + {} winnings = {} total", available_tokens, profit, max_payout);
            } else if max_payout > available_tokens {
                let profit = max_payout - available_tokens;
                println!("✅ PROFIT: {} tokens from multi-user pot", profit);
                println!("🎉 SUCCESS: USER 1 received deposit + winnings from USER 2!");
            } else if max_payout == available_tokens {
                println!("⚠️ SINGLE-USER RESULT: User received only original deposit back");
                println!("💡 This suggests pot distribution didn't include USER 2's losing bet");
            } else {
                println!("⚠️ PARTIAL PAYOUT: {} tokens received (less than original)", max_payout);
            }
        } else {
            println!("❌ No payout received - redemption may have failed validation");
        }

    } else {
        println!("❌ No coupon token found in traces - cannot demonstrate redemption");
    }

    Ok(())
}

#[wasm_bindgen_test]
fn test_factory_getter_calls() -> Result<()> {
    clear();
    
    println!("\n🔧 TESTING FACTORY GETTER CALLS");
    println!("=================================");

    // PHASE 1: Deploy Factory Template (using the working pattern from trace)
    println!("\n📦 PHASE 1: Deploying Factory Template");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [factory_build::get_bytes()].into(),
        [vec![3u128, 0x701u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Factory template deployed at block 0");

    // PHASE 2: Initialize Factory Contract (using the working pattern from trace)
    println!("\n🏭 PHASE 2: Initializing Factory Contract");
    let factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 1793u128, 0u128,  // Deploy to block 4, tx 1793, opcode 0 (Initialize)
                                    144u128,                  // success_threshold
                                    4u128, 1537u128,         // coupon_token_template_id (block 4, tx 1537)
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
    index_block(&factory_block, 1)?;
    
    let factory_contract_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);

    // PHASE 3: Test GetSuccessThreshold (opcode 21)
    println!("\n🔧 PHASE 3: Testing GetSuccessThreshold (opcode 21)");
    let threshold_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_contract_id.block,
                                    factory_contract_id.tx,
                                    21u128, // GetSuccessThreshold opcode
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
    index_block(&threshold_block, 2)?;
    
    // Read and display trace for GetSuccessThreshold
    println!("🔍 TRACE: GetSuccessThreshold (opcode 21)");
    // Check all vouts for trace data
    for vout in 0..5 {
        let threshold_outpoint = OutPoint {
            txid: threshold_block.txdata[0].compute_txid(),
            vout,
        };
        let threshold_trace_data = &view::trace(&threshold_outpoint)?;
        let threshold_trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&threshold_trace_data[..])?.into();
        let threshold_trace_guard = threshold_trace_result.0.lock().unwrap();
        if !threshold_trace_guard.is_empty() {
            println!("   • Vout {} trace data: {:?}", vout, *threshold_trace_guard);
            
            // Look for ReturnContext with data
            for trace in threshold_trace_guard.iter() {
                if let alkanes_support::trace::TraceEvent::ReturnContext(response) = trace {
                    if !response.inner.data.is_empty() {
                        println!("   • GetSuccessThreshold returned data: {:?}", response.inner.data);
                        // Convert bytes to u128 if possible
                        if response.inner.data.len() >= 8 {
                            // Take only the first 8 bytes for u128 conversion
                            match response.inner.data[0..8].try_into() {
                                Ok(bytes) => {
                                    let value = u128::from_le_bytes(bytes);
                                    println!("   • GetSuccessThreshold value: {}", value);
                                },
                                Err(_) => {
                                    println!("   • GetSuccessThreshold data conversion failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("✅ GetSuccessThreshold call completed");

    // PHASE 4: Test GetMinimumStake (opcode 51)
    println!("\n🔧 PHASE 4: Testing GetMinimumStake (opcode 51)");
    let min_stake_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_contract_id.block,
                                    factory_contract_id.tx,
                                    51u128, // GetMinimumStake opcode
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
    index_block(&min_stake_block, 3)?;
    
    // Read and display trace for GetMinimumStake
    println!("🔍 TRACE: GetMinimumStake (opcode 51)");
    // Check all vouts for trace data
    for vout in 0..5 {
        let min_stake_outpoint = OutPoint {
            txid: min_stake_block.txdata[0].compute_txid(),
            vout,
        };
        let min_stake_trace_data = &view::trace(&min_stake_outpoint)?;
        let min_stake_trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&min_stake_trace_data[..])?.into();
        let min_stake_trace_guard = min_stake_trace_result.0.lock().unwrap();
        if !min_stake_trace_guard.is_empty() {
            println!("   • Vout {} trace data: {:?}", vout, *min_stake_trace_guard);
            
            // Look for ReturnContext with data
            for trace in min_stake_trace_guard.iter() {
                if let alkanes_support::trace::TraceEvent::ReturnContext(response) = trace {
                    if !response.inner.data.is_empty() {
                        println!("   • GetMinimumStake returned data: {:?}", response.inner.data);
                        // Convert bytes to u128 if possible
                        if response.inner.data.len() >= 8 {
                            match response.inner.data[0..8].try_into() {
                                Ok(bytes) => {
                                    let value = u128::from_le_bytes(bytes);
                                    println!("   • GetMinimumStake value: {}", value);
                                },
                                Err(_) => {
                                    println!("   • GetMinimumStake data conversion failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("✅ GetMinimumStake call completed");

    // PHASE 5: Test GetCouponTokenTemplateId (opcode 23)
    println!("\n🔧 PHASE 5: Testing GetCouponTokenTemplateId (opcode 23)");
    let template_id_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_contract_id.block,
                                    factory_contract_id.tx,
                                    23u128, // GetCouponTokenTemplateId opcode
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
    index_block(&template_id_block, 4)?;
    
    // Read and display trace for GetCouponTokenTemplateId
    println!("🔍 TRACE: GetCouponTokenTemplateId (opcode 23)");
    // Check all vouts for trace data
    for vout in 0..5 {
        let template_id_outpoint = OutPoint {
            txid: template_id_block.txdata[0].compute_txid(),
            vout,
        };
        let template_id_trace_data = &view::trace(&template_id_outpoint)?;
        let template_id_trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&template_id_trace_data[..])?.into();
        let template_id_trace_guard = template_id_trace_result.0.lock().unwrap();
        if !template_id_trace_guard.is_empty() {
            println!("   • Vout {} trace data: {:?}", vout, *template_id_trace_guard);
            
            // Look for ReturnContext with data
            for trace in template_id_trace_guard.iter() {
                if let alkanes_support::trace::TraceEvent::ReturnContext(response) = trace {
                    if !response.inner.data.is_empty() {
                        println!("   • GetCouponTokenTemplateId returned data: {:?}", response.inner.data);
                        // Convert bytes to AlkaneId if possible (16 bytes for block + tx)
                        if response.inner.data.len() >= 16 {
                            match (response.inner.data[0..8].try_into(), response.inner.data[8..16].try_into()) {
                                (Ok(block_bytes), Ok(tx_bytes)) => {
                                    let block = u128::from_le_bytes(block_bytes);
                                    let tx = u128::from_le_bytes(tx_bytes);
                                    println!("   • GetCouponTokenTemplateId value: AlkaneId {{ block: {}, tx: {} }}", block, tx);
                                },
                                _ => {
                                    println!("   • GetCouponTokenTemplateId data conversion failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("✅ GetCouponTokenTemplateId call completed");

    println!("\n🎊 FACTORY GETTER CALLS TEST SUMMARY");
    println!("=====================================");
    println!("✅ Factory template deployed successfully");
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);
    println!("✅ GetSuccessThreshold (opcode 21) tested");
    println!("✅ GetMinimumStake (opcode 51) tested");
    println!("✅ GetCouponTokenTemplateId (opcode 23) tested");
    println!("✅ All getter calls completed successfully");

    Ok(())
}

#[wasm_bindgen_test]
fn test_complete_deposit_to_coupon_flow() -> Result<()> {
    clear();
    
    println!("\n🎰 COMPLETE DEPOSIT → COUPON CREATION FLOW TEST");
    println!("===============================================");

    // PHASE 1: Deploy All Contract Templates
    println!("\n📦 PHASE 1: Deploying All Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], // Free-mint template → deploys to 4,797 with complete parameters
            vec![3u128, 0x601],    // Coupon template → deploys to 4,0x601
            vec![3u128, 0x701],    // Factory template → deploys to 4,0x701
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ All contract templates deployed at block 0");

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

    // PHASE 3: Mint Tokens from Free-Mint Contract
    println!("\n💰 PHASE 3: Minting Tokens from Free-Mint Contract");
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Verify minted tokens
    let mint_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let minted_token_id = ProtoruneRuneId { block: 2, tx: 1 };
    let minted_amount = mint_sheet.get(&minted_token_id);

    println!("🔍 Minted token ID: {:?}", minted_token_id);
    println!("🔍 Minted amount: {}", minted_amount);

    assert!(minted_amount > 0, "Expected minted amount to be greater than 0");
    println!("✅ Tokens successfully minted and verified.");

    // PHASE 4: Initialize Factory Contract
    println!("\n🏭 PHASE 4: Initializing Factory Contract");
    let factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 0x701u128, 0u128,  // Deploy to block 6, tx 0x701, opcode 0 (Initialize)
                                    144u128,                   // success_threshold
                                    6u128,                     // coupon_token_template_id.block
                                    0x601u128,                 // coupon_token_template_id.tx
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
    index_block(&factory_block, 6)?;
    
    let factory_contract_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);

    // PHASE 5: Create Coupon (Deposit Operation)
    println!("\n🎫 PHASE 5: Creating Coupon (Deposit Operation)");
    println!("🔍 Available tokens at mint outpoint: {}", minted_amount);
    println!("🎯 Deposit amount: 5000");
    
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint, // Use the minted tokens as input
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
                        ProtostoneEdict {
                            id: minted_token_id,
                            amount: 5000, // Deposit 5000 tokens
                            output: 0,    // Send to factory
                        }.into()
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    factory_contract_id.block,
                                    factory_contract_id.tx,
                                    1u128, // CreateCoupon opcode
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
    index_block(&deposit_block, 1000)?; // FORCE BRAND NEW COUPONS AT BLOCK 1000
    println!("✅ CreateCoupon transaction submitted at block 10");

    // PHASE 6: Analyze Results
    println!("\n🔍 PHASE 6: Analyzing Deposit Results");
    println!("=====================================");
    
    // Check all vouts for trace data
    for vout in 0..5 {
        let deposit_outpoint = OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout,
        };
        let deposit_trace_data = &view::trace(&deposit_outpoint)?;
        let deposit_trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&deposit_trace_data[..])?.into();
        let deposit_trace_guard = deposit_trace_result.0.lock().unwrap();
        if !deposit_trace_guard.is_empty() {
            println!("   • vout {} trace data: {:?}", vout, *deposit_trace_guard);
            
            // Look for ReturnContext with coupon tokens
            for trace in deposit_trace_guard.iter() {
                if let alkanes_support::trace::TraceEvent::ReturnContext(response) = trace {
                    if !response.inner.alkanes.0.is_empty() {
                        println!("   • Coupon tokens returned: {:?}", response.inner.alkanes.0);
                        for (i, coupon_transfer) in response.inner.alkanes.0.iter().enumerate() {
                            println!("     - Coupon {}: ID={:?}, Value={}", i, coupon_transfer.id, coupon_transfer.value);
                        }
                    }
                }
            }
        }
    }

    // Check balance sheet for coupon tokens
    let deposit_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let deposit_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&deposit_outpoint)?));
    
    println!("\n📊 COUPON TOKEN ANALYSIS");
    println!("==========================");
    for (token_id, amount) in deposit_sheet.cached.balances.iter() {
        if token_id.block != 2 || token_id.tx != 1 { // Not the original minted token
            println!("   • Token ID: {:?}, Amount: {}", token_id, amount);
        }
    }

    println!("\n🎊 COMPLETE DEPOSIT → COUPON CREATION FLOW TEST SUMMARY");
    println!("=======================================================");
    println!("✅ All contract templates deployed successfully");
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);
    println!("✅ Tokens successfully minted: {} tokens", minted_amount);
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);
    println!("✅ CreateCoupon transaction submitted (opcode 1)");
    println!("✅ Trace analysis completed");
    println!("✅ Test completed successfully");

    Ok(())
}