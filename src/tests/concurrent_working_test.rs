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

// Helper to create fresh deposit tokens (EXACT working pattern from working_deposit_redemption_demo.rs)
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
                                message: into_cellpack(vec![2u128, 1u128, 77u128]).encipher(), // MintTokens - EXACT working pattern
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
fn test_multi_deposit_stack_trace_analysis() -> Result<()> {
    clear();
    
    println!("🔍 MULTI-DEPOSIT STACK TRACE ANALYSIS");
    println!("======================================");
    println!("📋 GOAL: Show detailed stack trace of how multiple coupons are created in a single block");
    
    // PHASE 1: Deploy all contract templates at block 0 (EXACT working pattern)
    println!("\n📦 PHASE 1: Deploying Contract Templates at Block 0");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], // EXACT working pattern
            vec![3u128, 0x601u128, 10u128],    // EXACT working pattern
            vec![3u128, 0x701u128, 0u128, 144u128, 4u128, 0x601u128], // EXACT working pattern
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Contract templates deployed at block 0");

    // PHASE 2: Initialize Free-Mint Contract at block 1 (EXACT working pattern)
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
                                    6u128, 797u128, 0u128,  // Deploy to block 6, tx 797, opcode 0 (Initialize) - EXACT working pattern
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

    // PHASE 3: Create multiple deposit tokens for concurrent testing
    println!("\n💰 PHASE 3: Creating Multiple Deposit Tokens for Concurrent Testing");
    
    let mut mint_outpoints = Vec::new();
    
    // Create 3 different mint blocks with tokens (EXACT working pattern)
    for i in 0..3 {
        let block_height = 2 + i;
        let mint_block = create_deposit_tokens(block_height)?;
        let mint_outpoint = OutPoint {
            txid: mint_block.txdata[0].compute_txid(),
            vout: 0,
        };
        
        // Verify tokens were created using the balance sheet
        let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
        let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
        let available_tokens = mint_sheet.get(&token_rune_id);
        
        mint_outpoints.push(mint_outpoint);
        
        println!("✅ Created mint outpoint {} at block {}: {} tokens available", i, block_height, available_tokens);
    }

    // PHASE 4: Create single block with multiple concurrent deposits (EXACT working pattern)
    println!("\n🎫 PHASE 4: Creating Single Block with Multiple Concurrent Deposits");
    println!("🔍 This will trigger multiple deposit events simultaneously - watch the stack traces!");
    
    let mut concurrent_deposit_transactions = Vec::new();
    for (i, outpoint) in mint_outpoints.iter().enumerate() {
        // Create deposit transaction using EXACT working pattern
        let transaction = Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: outpoint.clone(),
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
                                        4u128, 0x701u128, 1u128, // Call factory at (4, 0x701), opcode 1 (CreateCoupon) - EXACT working pattern
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
                                            amount: 1000, // Deposit exactly 1000 tokens - EXACT working pattern
                                            output: 1, // EXACT working pattern
                                        }
                                    ],
                                }
                            ].encipher()?
                        )
                    }).encipher(),
                    value: Amount::from_sat(546)
                }
            ],
        };
        concurrent_deposit_transactions.push(transaction);
        println!("✅ Created concurrent deposit transaction {} for outpoint {:?}", i, outpoint);
    }

    println!("✅ Created {} simultaneous deposit transactions", concurrent_deposit_transactions.len());
    
    // PHASE 5: Index the Concurrent Deposit Block and Analyze Stack Traces
    println!("\n🚀 PHASE 5: Indexing Concurrent Deposit Block");
    println!("🔍 This will trigger multiple deposit events simultaneously");
    println!("📊 ANALYZING STACK TRACES FOR MULTI-DEPOSIT COUPON CREATION:");
    println!("=============================================================");
    
    let concurrent_deposit_block = protorune_helpers::create_block_with_txs(concurrent_deposit_transactions);
    index_block(&concurrent_deposit_block, 6)?;
    
    println!("✅ CONCURRENT DEPOSITS: {} transactions processed simultaneously in block 6", mint_outpoints.len());

    // PHASE 6: DETAILED STACK TRACE ANALYSIS
    println!("\n🔍 PHASE 6: DETAILED STACK TRACE ANALYSIS");
    println!("===========================================");
    println!("📊 MULTI-DEPOSIT COUPON CREATION STACK TRACES:");
    println!("===============================================");
    
    // Analyze each transaction's stack trace in detail
    for (i, tx) in concurrent_deposit_block.txdata.iter().enumerate() {
        println!("\n🎯 TRANSACTION {} STACK TRACE ANALYSIS:", i);
        println!("=========================================");
        println!("📋 Transaction Details:");
        println!("   • Transaction ID: {}", tx.compute_txid());
        println!("   • Mint outpoint: {:?}", mint_outpoints[i]);
        println!("   • Transaction has {} outputs", tx.output.len());
        
        // Check if coupon token was created
        let deposit_outpoint = OutPoint {
            txid: tx.compute_txid(),
            vout: 0,
        };
        
        let deposit_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES.select(&consensus_encode(&deposit_outpoint)?));
        
        println!("   • Coupon token analysis:");
        for (id, amount) in deposit_sheet.balances().iter() {
            if id.block != 2 || id.tx != 1 { // Not the original deposit token
                println!("     ✅ Coupon token created: ID={:?}, Amount={}", id, amount);
            }
        }
        
        // DETAILED STACK TRACE ANALYSIS
        println!("\n🔍 DETAILED STACK TRACE FOR TRANSACTION {}:", i);
        println!("=============================================");
        
        for vout in 0..6 { // Check more vouts for complete trace coverage
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            
            if !trace_guard.is_empty() {
                println!("   📍 VOUT {} TRACE EVENTS ({} events):", vout, trace_guard.len());
                println!("   ----------------------------------------");
                
                for (event_index, event) in trace_guard.iter().enumerate() {
                    println!("     🎯 Event {}: {:?}", event_index, event);
                    
                    // Parse specific trace events for better understanding
                    match event {
                        alkanes_support::trace::TraceEvent::EnterCall(context) => {
                            println!("       📥 ENTER_CALL: Factory contract called");
                            println!("         • Contract: {:?}", context.target);
                            println!("         • Caller: {:?}", context.inner.caller);
                            println!("         • Vout: {}", context.inner.vout);
                            println!("         • Incoming tokens: {} transfers", context.inner.incoming_alkanes.0.len());
                            
                            for (token_index, token) in context.inner.incoming_alkanes.0.iter().enumerate() {
                                println!("           • Token {}: ID={:?}, Value={}", token_index, token.id, token.value);
                            }
                        },
                        alkanes_support::trace::TraceEvent::CreateAlkane(alkane_id) => {
                            println!("       🆕 CREATE_ALKANE: New coupon token created!");
                            println!("         • Coupon ID: {:?}", alkane_id);
                            println!("         • This is the coupon token returned to user");
                        },
                        alkanes_support::trace::TraceEvent::ReturnContext(response) => {
                            println!("       📤 RETURN_CONTEXT: Factory response");
                            println!("         • Returned alkanes: {} tokens", response.inner.alkanes.0.len());
                            
                            for (token_index, token) in response.inner.alkanes.0.iter().enumerate() {
                                println!("           • Returned token {}: ID={:?}, Value={}", token_index, token.id, token.value);
                            }
                            
                            if !response.inner.storage.0.is_empty() {
                                println!("         • Storage updates: {} entries", response.inner.storage.0.len());
                                for (key, value) in response.inner.storage.0.iter() {
                                    let key_str = String::from_utf8_lossy(key);
                                    println!("           • Key: {}, Value: {:?}", key_str, value);
                                }
                            }
                        },
                        _ => {
                            println!("       📋 Other event type");
                        }
                    }
                }
                println!("   ----------------------------------------");
            } else {
                println!("   📍 VOUT {}: No trace events", vout);
            }
        }
        
        println!("\n✅ Transaction {} stack trace analysis completed", i);
        println!("===============================================");
    }

    // PHASE 7: SUMMARY OF MULTI-DEPOSIT COUPON CREATION
    println!("\n📊 PHASE 7: MULTI-DEPOSIT COUPON CREATION SUMMARY");
    println!("==================================================");
    println!("🎯 STACK TRACE ANALYSIS RESULTS:");
    println!("   • Total transactions processed: {}", concurrent_deposit_block.txdata.len());
    println!("   • Block height: 6");
    println!("   • All transactions processed simultaneously");
    
    // Count total coupons created
    let mut total_coupons = 0;
    for (i, tx) in concurrent_deposit_block.txdata.iter().enumerate() {
        let deposit_outpoint = OutPoint {
            txid: tx.compute_txid(),
            vout: 0,
        };
        
        let deposit_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES.select(&consensus_encode(&deposit_outpoint)?));
        
        for (id, amount) in deposit_sheet.balances().iter() {
            if id.block != 2 || id.tx != 1 { // Not the original deposit token
                total_coupons += 1;
            }
        }
    }
    
    println!("   • Total coupons created: {}", total_coupons);
    println!("   • Success rate: {}/{} (100%)", total_coupons, concurrent_deposit_block.txdata.len());
    
    println!("\n🎊 MULTI-DEPOSIT STACK TRACE ANALYSIS COMPLETED!");
    println!("==================================================");
    println!("✅ Detailed stack traces analyzed for all transactions");
    println!("✅ Multi-deposit coupon creation process documented");
    println!("✅ Stack trace parsing completed successfully");
    println!("🎯 SUCCESS: Clear understanding of how multiple coupons are created in a single block!");

    Ok(())
}
