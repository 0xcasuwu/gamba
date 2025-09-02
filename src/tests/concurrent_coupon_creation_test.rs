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

/// Helper function to create a mint outpoint with specific token amount
fn create_mint_outpoint_with_amount(block_height: u32, token_amount: u128, free_mint_id: &AlkaneId) -> Result<(Block, OutPoint)> {
    let mint_block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                    edicts: vec![ProtostoneEdict {
                        id: ProtoruneRuneId { 
                            block: free_mint_id.block, 
                            tx: free_mint_id.tx 
                        },
                        amount: token_amount,
                        output: 0,
                    }.into()],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    2u128, 1u128, 77u128, // Call free-mint contract at block 2, tx 1, opcode 77 (MintTokens)
                                    0u128, // Additional parameters
                                ]).encipher(),
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

    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 1, // Token output is at vout 1
    };

    Ok((mint_block, mint_outpoint))
}

/// Helper function to create a coupon creation transaction for a specific mint outpoint
fn create_coupon_creation_transaction(
    factory_id: &AlkaneId,
    mint_outpoint: &OutPoint,
    stake_amount: u128,
    block_height: u32,
    free_mint_id: &AlkaneId
) -> Result<Transaction> {
    Ok(Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint.clone(), // Use the actual mint outpoint
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
                    edicts: vec![ProtostoneEdict {
                        id: ProtoruneRuneId { 
                            block: free_mint_id.block, 
                            tx: free_mint_id.tx 
                        },
                        amount: stake_amount,
                        output: 0, // Send to output 0 (the alkane call)
                    }.into()],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    2u128, 0x701u128, 1u128, // CreateCoupon opcode - factory at block 2, tx 0x701
                                ]).encipher(),
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
    })
}

#[wasm_bindgen_test]
fn test_concurrent_coupon_creation() -> Result<()> {
    clear();
    println!("\n🚀 CONCURRENT COUPON CREATION TEST");
    println!("====================================");
    println!("📋 GOAL: Demonstrate multiple unique mint outpoints processed simultaneously");
    println!("         in a single block with multiple coupon creation events");

    // PHASE 1: Deploy All Contract Templates
    println!("\n📦 PHASE 1: Deploying All Contract Templates");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx([
        free_mint_build::get_bytes(),
        coupon_template_build::get_bytes(),
        factory_build::get_bytes(),
    ].into(), [
        vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], // Deploy free-mint to block 4
        vec![3u128, 0x601u128, 10u128], // Deploy coupon template to block 4
        vec![3u128, 0x701u128, 0u128, 144u128, 4u128, 0x601u128], // Deploy factory template to block 4
    ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>());
    index_block(&template_block, 0)?;
    println!("✅ All contract templates deployed at block 0");

    // PHASE 2: Initialize Free-Mint Contract using 6→2 pattern
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract");
    let free_mint_block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 797u128, 0u128,  // Call 6,797,0 → spawns at block 2
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
                                edicts: vec![], // No tokens needed for getter queries
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&free_mint_block, 1)?;
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 }; // Contract spawns at block 2
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);

    // PHASE 3: Initialize Factory Contract using 6→2 pattern
    println!("\n🏭 PHASE 3: Initializing Factory Contract");
    let factory_init_block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 0x701u128, 0u128,  // Call 6,0x701,0 → spawns at block 2
                                    144u128,                // success_threshold
                                    4u128, 0x601u128,      // coupon_template_id
                                ]).encipher(),
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
    index_block(&factory_init_block, 2)?;
    let factory_id = AlkaneId { block: 2, tx: 0x701 }; // Factory spawns at block 2
    println!("✅ Factory contract initialized at {:?}", factory_id);

    // PHASE 4: Create Multiple Unique Mint Outpoints with Different Token Amounts
    println!("\n💰 PHASE 4: Creating Multiple Unique Mint Outpoints");
    
    // Create 5 different mint outpoints with varying token amounts
    let mint_configs = vec![
        (1000u128, "Alice"),
        (2500u128, "Bob"), 
        (5000u128, "Charlie"),
        (7500u128, "Diana"),
        (10000u128, "Eve"),
    ];

    let mut mint_outpoints = Vec::new();
    let mut mint_blocks = Vec::new();

    for (i, (amount, name)) in mint_configs.iter().enumerate() {
        let (mint_block, mint_outpoint) = create_mint_outpoint_with_amount(7 + i as u32, *amount, &free_mint_contract_id)?;
        mint_blocks.push(mint_block);
        mint_outpoints.push(mint_outpoint);
        
        println!("✅ Created mint outpoint for {}: {} tokens at {:?}", name, amount, mint_outpoint);
    }

    // PHASE 5: Index All Mint Blocks to Create Token Balances
    println!("\n📊 PHASE 5: Indexing Mint Blocks to Create Token Balances");
    for (i, mint_block) in mint_blocks.iter().enumerate() {
        index_block(mint_block, 7 + i as u32)?;
        println!("✅ Indexed mint block {} for {}", 7 + i, mint_configs[i].1);
    }

    // PHASE 6: Create Single Block with Multiple Coupon Creation Events
    println!("\n🎫 PHASE 6: Creating Single Block with Multiple Coupon Creation Events");
    
    // Create a single block containing multiple coupon creation transactions
    let concurrent_coupon_block = protorune_helpers::create_block_with_txs(
        mint_outpoints.iter().enumerate().map(|(i, outpoint)| {
            create_coupon_creation_transaction(
                &factory_id,
                outpoint,
                mint_configs[i].0, // Use the same amount as minted
                20, // All transactions in block 20
                &free_mint_contract_id // Pass the free mint contract ID
            ).unwrap()
        }).collect()
    );

    println!("✅ Created block 20 with {} simultaneous coupon creation transactions", mint_outpoints.len());
    
    // PHASE 7: Index the Concurrent Coupon Creation Block
    println!("\n🚀 PHASE 7: Indexing Concurrent Coupon Creation Block");
    println!("🔍 This will trigger multiple coupon creation events simultaneously");
    println!("📋 Expecting to see stack traces for concurrent processing");
    
    index_block(&concurrent_coupon_block, 20)?;
    
    println!("✅ Concurrent coupon creation block indexed successfully!");

    // PHASE 8: Analyze Results and Stack Traces
    println!("\n🔍 PHASE 8: Analyzing Results and Stack Traces");
    println!("=============================================");
    
    println!("📊 CONCURRENT BLOCK ANALYSIS:");
    println!("   • Block height: 20");
    println!("   • Total transactions: {}", concurrent_coupon_block.txdata.len());
    println!("   • Expected coupon creations: {}", mint_outpoints.len());
    
    // Analyze each transaction's trace by creating outpoints
    for (i, tx) in concurrent_coupon_block.txdata.iter().enumerate() {
        if i == 0 { continue; } // Skip coinbase transaction
        
        println!("\n🔍 Transaction {} ({}):", i, mint_configs[i-1].1);
        println!("   • Stake amount: {} tokens", mint_configs[i-1].0);
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

    println!("\n🎊 CONCURRENT COUPON CREATION TEST COMPLETED!");
    println!("=============================================");
    println!("✅ Multiple unique mint outpoints created successfully");
    println!("✅ All mint outpoints indexed to create token balances");
    println!("✅ Single block created with {} simultaneous coupon creation events", mint_outpoints.len());
    println!("✅ Concurrent processing completed - check stack traces above");
    println!("🔍 Stack trace analysis shows how multiple coupon creations were processed");

    Ok(())
}
