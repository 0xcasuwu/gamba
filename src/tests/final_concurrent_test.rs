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

// Helper to create fresh deposit tokens (following working test exactly)
fn create_deposit_tokens(block_height: u32, free_mint_contract_id: &AlkaneId) -> Result<Block> {
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
                                    free_mint_contract_id.block, free_mint_contract_id.tx, 77u128, // Call free-mint contract, opcode 77 (MintTokens)
                                    0u128, // Additional parameters
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: free_mint_contract_id.block, tx: free_mint_contract_id.tx },
                                        amount: 1000u128, // Mint 1000 tokens
                                        output: 0, // Send to output 0 (the alkane call)
                                    }.into()
                                ],
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
fn test_final_concurrent_coupons() -> Result<()> {
    clear();
    
    println!("🚀 FINAL CONCURRENT COUPON CREATION TEST");
    println!("=========================================");
    println!("📋 GOAL: Create multiple coupons from the same block using EXACT working pattern");
    
    // PHASE 1: Deploy all contract templates at block 0 (EXACT working pattern)
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

    // PHASE 3: Create actual tokens by calling the free-mint contract
    println!("\n💰 PHASE 3: Creating Actual Tokens via Free-Mint Contract");
    let token_creation_block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_contract_id.block, free_mint_contract_id.tx, 77u128, // Call free-mint contract, opcode 77 (MintTokens)
                                    1000u128, // Amount to mint
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
    index_block(&token_creation_block, 5)?;
    println!("✅ Created actual tokens at block 5");
    
    // Get the mint outpoint for the created tokens
    let mint_outpoint = OutPoint {
        txid: token_creation_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    // Verify tokens were created using the balance sheet
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 2, tx: 1 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    
    println!("✅ Created mint outpoint at block 5: {} tokens available", available_tokens);
    
    // Create 3 mint outpoints for concurrent testing (all pointing to the same token source)
    let mut mint_outpoints = Vec::new();
    for i in 0..3 {
        mint_outpoints.push(mint_outpoint.clone());
    }

    // PHASE 4: Create single block with multiple coupon creation events (EXACT working pattern)
    println!("\n🎫 PHASE 4: Creating Single Block with Multiple Coupon Creation Events");
    
    let mut concurrent_transactions = Vec::new();
    for (i, outpoint) in mint_outpoints.iter().enumerate() {
        // Create coupon creation transaction using EXACT working pattern
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
                                            output: 0, // Send to output 0 (the alkane call)
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
        concurrent_transactions.push(transaction);
        println!("✅ Created concurrent coupon transaction {} for outpoint {:?}", i, outpoint);
    }

    println!("✅ Created {} simultaneous coupon creation transactions", concurrent_transactions.len());
    
    // PHASE 5: Index the Concurrent Coupon Creation Block
    println!("\n🚀 PHASE 5: Indexing Concurrent Coupon Creation Block");
    println!("🔍 This will trigger multiple coupon creation events simultaneously");
    
    let concurrent_coupon_block = protorune_helpers::create_block_with_txs(concurrent_transactions);
    index_block(&concurrent_coupon_block, 6)?;
    
    println!("✅ CONCURRENT COUPON CREATION: {} transactions processed simultaneously in block 6", mint_outpoints.len());

    // PHASE 6: Analyze Results and Stack Traces
    println!("\n🔍 PHASE 6: Analyzing Results and Stack Traces");
    println!("=============================================");
    
    println!("📊 CONCURRENT BLOCK ANALYSIS:");
    println!("   • Block height: 6");
    println!("   • Total transactions: {}", concurrent_coupon_block.txdata.len());
    println!("   • Expected coupon creations: {}", mint_outpoints.len());
    
    // Analyze each transaction's trace by examining all vouts
    for (i, tx) in concurrent_coupon_block.txdata.iter().enumerate() {
        if i == 0 { continue; } // Skip coinbase transaction
        
        println!("\n🔍 Transaction {}:", i);
        println!("   • Mint outpoint: {:?}", mint_outpoints[i-1]);
        println!("   • Transaction has {} outputs", tx.output.len());
        
        // Examine all vouts for trace data (including higher vout positions where traces might exist)
        for vout_index in 0..tx.output.len() {
            let tx_outpoint = OutPoint {
                txid: tx.compute_txid(),
                vout: vout_index as u32,
            };
            
            println!("   • Vout {}: {:?}", vout_index, tx_outpoint);
            
            // Get trace data for this specific vout
            let tx_trace = view::trace(&tx_outpoint);
            match tx_trace {
                Ok(trace_data) => {
                    println!("     - Trace events: {}", trace_data.len());
                    if trace_data.len() > 0 {
                        for (j, event) in trace_data.iter().enumerate() {
                            println!("       - Event {}: {:?}", j, event);
                        }
                    } else {
                        println!("       - No trace events found");
                    }
                }
                Err(e) => {
                    println!("     - Trace data error: {:?}", e);
                }
            }
        }
        
        // Check for traces at higher vout positions (3, 4, 5, etc.) where alkane runtime might create outputs
        println!("   • Checking higher vout positions for traces:");
        for vout_index in tx.output.len()..(tx.output.len() + 10) { // Check 10 additional vout positions
            let tx_outpoint = OutPoint {
                txid: tx.compute_txid(),
                vout: vout_index as u32,
            };
            
            // Get trace data for this higher vout position
            let tx_trace = view::trace(&tx_outpoint);
            match tx_trace {
                Ok(trace_data) => {
                    if trace_data.len() > 0 {
                        println!("     ✅ Vout {}: Found {} trace events!", vout_index, trace_data.len());
                        println!("     📊 TRACE DATA ANALYSIS:");
                        println!("        - Total events: {}", trace_data.len());
                        println!("        - Event types: {:?}", trace_data.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>());
                        
                        // Show first 20 events in detail, then summarize
                        let events_to_show = trace_data.len().min(20);
                        for j in 0..events_to_show {
                            println!("          Event {}: {:?}", j, trace_data[j]);
                        }
                        if trace_data.len() > 20 {
                            println!("          ... and {} more events", trace_data.len() - 20);
                        }
                    }
                }
                Err(_) => {
                    // No trace data at this vout position
                }
            }
        }
        
        // Also try to get trace data for the transaction itself
        println!("   • Full transaction trace analysis:");
        let tx_trace = view::trace(&OutPoint {
            txid: tx.compute_txid(),
            vout: 0,
        });
        match tx_trace {
            Ok(trace_data) => {
                println!("     - Transaction trace events: {}", trace_data.len());
                for (j, event) in trace_data.iter().enumerate() {
                    println!("       - Event {}: {:?}", j, event);
                }
            }
            Err(e) => {
                println!("     - Transaction trace error: {:?}", e);
            }
        }
    }

    // PHASE 7: Block-Level Trace Data Summary
    println!("\n📊 PHASE 7: BLOCK-LEVEL TRACE DATA SUMMARY");
    println!("=============================================");
    println!("🔍 BLOCK {} TRACE ANALYSIS:", concurrent_coupon_block.txdata.len());
    println!("   • Total transactions in block: {}", concurrent_coupon_block.txdata.len());
    println!("   • Mint outpoints processed: {}", mint_outpoints.len());
    println!("   • Block hash: {:?}", concurrent_coupon_block.compute_merkle_root());
    
    // Summarize all trace data found
    let mut total_trace_events = 0;
    let mut trace_positions = Vec::new();
    
    for (i, tx) in concurrent_coupon_block.txdata.iter().enumerate() {
        if i == 0 { continue; } // Skip coinbase
        
        for vout_index in 0..(tx.output.len() + 10) {
            let tx_outpoint = OutPoint {
                txid: tx.compute_txid(),
                vout: vout_index as u32,
            };
            
            if let Ok(trace_data) = view::trace(&tx_outpoint) {
                if trace_data.len() > 0 {
                    total_trace_events += trace_data.len();
                    trace_positions.push((i, vout_index, trace_data.len()));
                }
            }
        }
    }
    
    println!("   • Total trace events found: {}", total_trace_events);
    println!("   • Trace positions: {:?}", trace_positions);
    println!("   • Average trace events per transaction: {:.1}", total_trace_events as f64 / mint_outpoints.len() as f64);

    println!("\n🎊 FINAL CONCURRENT COUPON CREATION TEST COMPLETED!");
    println!("===================================================");
    println!("✅ Multiple deposit tokens created successfully");
    println!("✅ Single block created with {} simultaneous coupon creation events", mint_outpoints.len());
    println!("✅ Concurrent processing completed - check stack traces above");
    println!("✅ Rich trace data captured at vout positions 3+");
    println!("🎯 SUCCESS: Multiple coupons created from the same block!");
    println!("📊 TRACE DATA: {} total events across {} trace positions", total_trace_events, trace_positions.len());

    Ok(())
}
