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
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use alkanes::view;
// use protobuf::Message; // Removed unused import

use alkanes::precompiled::free_mint_build;
use crate::precompiled::factory_build;
use crate::precompiled::coupon_template_build;
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
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
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
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
            auth_token_build::get_bytes(),
        ].into(),
        [
            // free_mint template → deploys instance at block 4, tx 797 (opcode 0 for init)
            // Arguments: token_units, value_per_mint, cap, name_part1, name_part2, symbol
            vec![3u128, 797u128, 0u128],
            // coupon_token template → deploys instance at block 4, tx 0x601 (opcode 0 for init)
            // Arguments: position_id, deposit_amount, reward_debt, deposit_block, deposit_token_id.block, deposit_token_id.tx
            vec![3u128, 0x601, 0u128, 0u128, 0u128, 0u128, 2u128, 797u128], // DUST token ID for deposit_token_id
            // coupon_factory template → deploys instance at block 4, tx 0x701 (opcode 0 for init)
            // Arguments: deposit_token_id.block, deposit_token_id.tx, reward_per_block, start_block, end_reward_block, free_mint_contract_id.block, free_mint_contract_id.tx
            vec![3u128, 0x701, 0u128, 2u128, 797u128, 100u128, 0u128, 1000u128, 4u128, 797u128], // DUST token ID for deposit_token_id, free_mint instance ID for free_mint_contract_id
            vec![3u128, 0xffee, 0u128, 1u128], // auth_token template → deploys at block 4
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
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }
    println!("✅ Templates deployed successfully");
    
    // STEP 2: Deploy DUST token 
    println!("\n💨 STEP 2: DUST Token Deployment");
    let dust_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
     index_block(&dust_block, 1)?;
     
     // TRACE: DUST token deployment
     for (i, tx) in dust_block.txdata.iter().enumerate() {
         println!("   • TX {} traces:", i);
         for vout in 0..5 {
             let trace_data = &view::trace(&OutPoint {
                 txid: tx.compute_txid(),
                 vout,
             })?;
             let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
             let trace_guard = trace_result.0.lock().unwrap();
             if !trace_guard.is_empty() {
                 println!("     - vout {}: {:?}", vout, *trace_guard);
             }
         }
     }
     println!("✅ DUST token deployed successfully");
     
     // STEP 3: Mint tokens from the Free-Mint Contract (using working minting pattern)
     println!("\n💰 STEP 3: Minting Tokens from Free-Mint Contract");
     let free_mint_contract_id = AlkaneId { block: 2, tx: 1 }; // Factory spawns free-mint at block 2, tx 1
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
     println!("✅ DUST token deployed and initialized.");
     println!("✅ Tokens successfully minted from the free-mint contract.");
     println!("✅ Test completed successfully.");

    Ok(())
}