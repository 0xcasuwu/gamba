#!/usr/bin/env python3

import os

def update_precompiled_module(wasm_file, output_file):
    """Read WASM file and create hex-encoded Rust module"""
    
    # Read the WASM file
    with open(wasm_file, 'rb') as f:
        wasm_bytes = f.read()
    
    # Convert to hex string
    hex_string = wasm_bytes.hex()
    
    # Create the Rust module content
    rust_content = f'''use hex_lit::hex;
#[allow(long_running_const_eval)]
pub fn get_bytes() -> Vec<u8> {{ (&hex!("{hex_string}")).to_vec() }}
'''
    
    # Write to output file
    with open(output_file, 'w') as f:
        f.write(rust_content)
    
    print(f"Updated {output_file} with {len(wasm_bytes)} bytes of WASM data (no bonus multipliers)")

def main():
    base_dir = "/Users/erickdelgado/Documents/GitHub/gamba"
    
    # Update factory build
    factory_wasm = os.path.join(base_dir, "target/wasm32-unknown-unknown/release/alkane_factory.wasm")
    factory_output = os.path.join(base_dir, "src/precompiled/factory_build.rs")
    
    if os.path.exists(factory_wasm):
        update_precompiled_module(factory_wasm, factory_output)
    else:
        print(f"Factory WASM file not found: {factory_wasm}")
    
    # Update coupon template build
    coupon_wasm = os.path.join(base_dir, "target/wasm32-unknown-unknown/release/alkane_coupon_template.wasm")
    coupon_output = os.path.join(base_dir, "src/precompiled/coupon_template_build.rs")
    
    if os.path.exists(coupon_wasm):
        update_precompiled_module(coupon_wasm, coupon_output)
    else:
        print(f"Coupon template WASM file not found: {coupon_wasm}")

if __name__ == "__main__":
    main()
