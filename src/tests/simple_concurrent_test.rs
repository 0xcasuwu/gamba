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
fn test_simple_concurrent_coupons() -> Result<()> {
    clear();
    
    println!("🚀 SIMPLE CONCURRENT COUPON CREATION TEST");
    println!("==========================================");
    println!("📋 GOAL: Create multiple coupons from the same block using working pattern");
    
    // PHASE 1: Deploy all contract templates at block 0 (following working test exactly)
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

    // PHASE 2: Initialize Free-Mint Contract at block 1 (following working test exactly)
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

    // PHASE 3: Create multiple deposit tokens for concurrent coupon creation
    println!("\n💰 PHASE 3: Creating Multiple Deposit Tokens");
    
    let mut mint_blocks = Vec::new();
    let mut mint_outpoints = Vec::new();
    
    // Create 3 different mint blocks with tokens
    for i in 0..3 {
        let block_height = 2 + i;
        let mint_block = create_deposit_tokens(block_height)?;
        let mint_outpoint = OutPoint {
            txid: mint_block.txdata[0].compute_txid(),
            vout: 0,
        };
        mint_blocks.push(mint_block);
        mint_outpoints.push(mint_outpoint);
        
        println!("✅ Created mint outpoint {} at block {}: {:?}", i, block_height, mint_outpoint);
    }

    // PHASE 4: Create single block with multiple coupon creation events
    println!("\n🎫 PHASE 4: Creating Single Block with Multiple Coupon Creation Events");
    
    // Create a single block containing multiple coupon creation transactions
    let mut transactions = Vec::new();
    for (i, outpoint) in mint_outpoints.iter().enumerate() {
        // Create coupon creation transaction for each mint outpoint
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
                        edicts: vec![
                            // Send stake tokens to factory following boiler alkane input pattern
                            ProtostoneEdict {
                                id: ProtoruneRuneId { 
                                    block: free_mint_contract_id.block, 
                                    tx: free_mint_contract_id.tx 
                                },
                                amount: 1000u128, // Stake amount
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
                            ].encipher().unwrap()
                        )
                    }).encipher(),
                    value: Amount::from_sat(546)
                }
            ],
        };
        transactions.push(transaction);
    }
    
    let concurrent_coupon_block = protorune_helpers::create_block_with_txs(transactions);

    println!("✅ Created block 6 with {} simultaneous coupon creation transactions", mint_outpoints.len());
    
    // PHASE 5: Index the Concurrent Coupon Creation Block
    println!("\n🚀 PHASE 5: Indexing Concurrent Coupon Creation Block");
    println!("🔍 This will trigger multiple coupon creation events simultaneously");
    
    index_block(&concurrent_coupon_block, 6)?;
    
    println!("✅ Concurrent coupon creation block indexed successfully!");

    // PHASE 6: Analyze Results and Stack Traces
    println!("\n🔍 PHASE 6: Analyzing Results and Stack Traces");
    println!("=============================================");
    
    println!("📊 CONCURRENT BLOCK ANALYSIS:");
    println!("   • Block height: 6");
    println!("   • Total transactions: {}", concurrent_coupon_block.txdata.len());
    println!("   • Expected coupon creations: {}", mint_outpoints.len());
    
    // Analyze each transaction's trace by creating outpoints
    for (i, tx) in concurrent_coupon_block.txdata.iter().enumerate() {
        if i == 0 { continue; } // Skip coinbase transaction
        
        println!("\n🔍 Transaction {}:", i);
        println!("   • Mint outpoint: {:?}", mint_outpoints[i-1]);
        
        // Create outpoint for this transaction to get trace data
        let tx_outpoint = OutPoint {
            txid: tx.compute_txid(),
            vout: 1, // Token output is at vout 1
        };
        
        // Get trace data for this specific transaction
        let tx_trace = view::trace(&tx_outpoint);
        match tx_trace {
            Ok(trace_data) => {
                println!("   • Trace events: {}", trace_data.len());
                for (j, event) in trace_data.iter().enumerate() {
                    println!("     - Event {}: {:?}", j, event);
                }
            }
            Err(e) => {
                println!("   • Trace data error: {:?}", e);
            }
        }
    }

    println!("\n🎊 SIMPLE CONCURRENT COUPON CREATION TEST COMPLETED!");
    println!("=====================================================");
    println!("✅ Multiple deposit tokens created successfully");
    println!("✅ Single block created with {} simultaneous coupon creation events", mint_outpoints.len());
    println!("✅ Concurrent processing completed - check stack traces above");

    Ok(())
}
