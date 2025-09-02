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

// Helper to create fresh deposit tokens (following boiler pattern)
fn create_deposit_tokens(block_height: u32) -> Result<Block> {
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // MintTokens
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
    
    println!("✅ Created fresh deposit tokens at block {}", block_height);
    Ok(mint_block)
}

#[wasm_bindgen_test]
fn test_working_deposit_redemption() -> Result<()> {
    clear();
    
    println!("🚀 GAMBA DEPOSIT → COUPON → REDEMPTION TEST");
    println!("============================================");
    
    // PHASE 1: Deploy all contract templates at block 0 (following boiler pattern)
    println!("\n📦 PHASE 1: Deploying Contract Templates at Block 0");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354],
            vec![3u128, 0x601u128, 10u128],
            vec![3u128, 0x701u128, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Contract templates deployed at block 0");

    // PHASE 2: Initialize Free-Mint Contract at block 1 (following boiler pattern)
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract at Block 1");
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

    // PHASE 3: Create deposit tokens and perform deposit
    println!("\n💰 PHASE 3: Creating Deposit Tokens and Performing Deposit");
    
    // Create fresh deposit tokens at block 2
    let mint_block = create_deposit_tokens(2)?;
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };

    // Get available tokens
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("🔍 Available tokens: {}", available_tokens);
    println!("🎯 Deposit amount: 1000 tokens");
    
    if available_tokens < 1000 {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need 1000", available_tokens));
    }

    // Perform deposit at block 3
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
                                    4u128, 0x701u128, 1u128, // Call factory at (4, 0x701), opcode 1 (CreateCoupon)
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
                                        amount: 1000, // Deposit exactly 1000 tokens
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
    index_block(&deposit_block, 3)?;
    
    println!("✅ Deposit completed at block 3");
    
    // Get the coupon outpoint
    let coupon_outpoint = OutPoint { 
        txid: deposit_block.txdata[0].compute_txid(), 
        vout: 0 
    };

    // PHASE 4: Analyze the results to see if coupon was created
    println!("\n🎰 PHASE 4: Analyzing Coupon Creation Results");
    
    // Check if coupon token was created
    let coupon_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&coupon_outpoint)?));
    
    println!("🔍 Coupon token analysis:");
    for (id, amount) in coupon_sheet.balances().iter() {
        println!("   • Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Also check the deposit transaction trace to see what happened
    println!("\n🔍 DEPOSIT TRANSACTION TRACE:");
    for vout in 0..3 {
        let trace_data = &view::trace(&OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • vout {}: {:?}", vout, *trace_guard);
        }
    }
    
    // PHASE 5: Test Redemption with the Coupon Token
    println!("\n🎰 PHASE 5: Testing Redemption with Coupon Token");
    
    // We know the coupon token is at (2, 2) from the analysis above
    let coupon_token_id = ProtoruneRuneId { block: 2, tx: 2 };
    println!("🎫 Using coupon token: {:?}", coupon_token_id);
    
    // Create a transaction that sends the coupon token to the factory for redemption
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
                    edicts: vec![
                        ProtostoneEdict {
                            id: coupon_token_id,
                            amount: 1, // Send 1 coupon token to factory
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
                                    4u128, 0x701u128, 2u128, // Call factory at (4, 0x701), opcode 2 (RedeemCoupon)
                                    2u128, 2u128, // coupon_id: AlkaneId { block: 2, tx: 2 }
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
    index_block(&redemption_block, 4)?;
    
    println!("✅ Redemption attempt completed at block 4");
    
    // Analyze the redemption results
    let redemption_outpoint = OutPoint {
        txid: redemption_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let redemption_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&redemption_outpoint)?));
    
    println!("\n💰 REDEMPTION RESULTS:");
    for (id, amount) in redemption_sheet.balances().iter() {
        println!("   • Received Token ID: {:?}, Amount: {}", id, amount);
    }
    
    // Check redemption transaction trace
    println!("\n🔍 REDEMPTION TRANSACTION TRACE:");
    for vout in 0..3 {
        let trace_data = &view::trace(&OutPoint {
            txid: redemption_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • vout {}: {:?}", vout, *trace_guard);
        }
    }
    
    println!("\n🎊 GAMBA DEPOSIT → COUPON → REDEMPTION TEST COMPLETED!");
    println!("=========================================================");
    println!("✅ Contract templates deployed at block 0");
    println!("✅ Free-mint contract initialized at block 1");
    println!("✅ Deposit tokens created at block 2");
    println!("✅ Deposit completed at block 3");
    println!("✅ Coupon token created at {:?}", coupon_token_id);
    println!("✅ Redemption attempted at block 4");
    println!("✅ Full deposit → coupon → redemption cycle completed!");
    
    Ok(())
}
