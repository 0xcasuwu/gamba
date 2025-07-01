use alkanes_support::id::AlkaneId;
use bitcoin::{Txid, blockdata::block::TxMerkleNode};
use anyhow::Result;

pub struct WandData {
    pub wand_id: u128,
    pub position_token_id: AlkaneId,
    pub txid: Txid,
    pub merkle_root: TxMerkleNode,
    pub base_xor_result: u8,
    pub dust_bonus: u8,
    pub final_xor_result: u8,
    pub dust_amount: u128,
}

pub struct WandSvgGenerator;

impl WandSvgGenerator {
    pub fn generate_svg(data: WandData) -> Result<String> {
        let wand_power = Self::calculate_wand_power(data.final_xor_result);
        let wand_type = Self::get_wand_type(data.final_xor_result);
        let magical_color = Self::get_magical_color(data.final_xor_result);
        let position_display = format!("{}:{}", data.position_token_id.block, data.position_token_id.tx);
        let txid_short = format!("{}...{}", 
            &data.txid.to_string()[0..8], 
            &data.txid.to_string()[56..64]
        );
        let merkle_short = format!("{}...{}", 
            &data.merkle_root.to_string()[0..8], 
            &data.merkle_root.to_string()[56..64]
        );
        let dust_display = Self::format_dust_amount(data.dust_amount);

        let svg = format!(r##"<svg viewBox="0 0 300 450" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <!-- Magical gradient backgrounds -->
    <linearGradient id="wandGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#0f0f23;stop-opacity:1" />
      <stop offset="25%" style="stop-color:#1a1a3a;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#2d1b69;stop-opacity:1" />
      <stop offset="75%" style="stop-color:#4c1d95;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#581c87;stop-opacity:1" />
    </linearGradient>
    
    <radialGradient id="orbitalGold" cx="50%" cy="50%" r="50%">
      <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="30%" style="stop-color:#f59e0b;stop-opacity:1" />
      <stop offset="70%" style="stop-color:#d97706;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#92400e;stop-opacity:1" />
    </radialGradient>
    
    <linearGradient id="wandCore" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#8b5cf6;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#a78bfa;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#c4b5fd;stop-opacity:1" />
    </linearGradient>
    
    <radialGradient id="magicalOrb" cx="50%" cy="50%" r="50%">
      <stop offset="0%" style="stop-color:{};stop-opacity:1" />
      <stop offset="50%" style="stop-color:#8b5cf6;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#4c1d95;stop-opacity:1" />
    </radialGradient>
    
    <linearGradient id="cosmicGlow" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="25%" style="stop-color:#34d399;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#60a5fa;stop-opacity:1" />
      <stop offset="75%" style="stop-color:#a78bfa;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#fbbf24;stop-opacity:1" />
    </linearGradient>
    
    <linearGradient id="dustGlow" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#d4af37;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#ffd700;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#b8860b;stop-opacity:1" />
    </linearGradient>
    
    <!-- Magical filters and effects -->
    <filter id="orbitalGlow">
      <feGaussianBlur stdDeviation="8" result="coloredBlur"/>
      <feMerge> 
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <filter id="wandShimmer">
      <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
      <feDropShadow dx="2" dy="2" stdDeviation="6" flood-color="#a78bfa" flood-opacity="0.6"/>
      <feMerge> 
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <filter id="dustShimmer">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feDropShadow dx="1" dy="1" stdDeviation="4" flood-color="#ffd700" flood-opacity="0.8"/>
      <feMerge> 
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <!-- Animated sparkle pattern -->
    <pattern id="orbitalPattern" patternUnits="userSpaceOnUse" width="40" height="40">
      <circle cx="10" cy="10" r="1" fill="#fbbf24" opacity="0.4">
        <animate attributeName="opacity" values="0.4;0.8;0.4" dur="3s" repeatCount="indefinite"/>
      </circle>
      <circle cx="30" cy="30" r="0.8" fill="#a78bfa" opacity="0.3">
        <animate attributeName="opacity" values="0.3;0.6;0.3" dur="4s" repeatCount="indefinite"/>
      </circle>
      <circle cx="20" cy="5" r="0.6" fill="#34d399" opacity="0.35">
        <animate attributeName="opacity" values="0.35;0.7;0.35" dur="2.5s" repeatCount="indefinite"/>
      </circle>
    </pattern>
  </defs>
  
  <!-- Cosmic background with orbital movement -->
  <rect x="0" y="0" width="300" height="450" fill="url(#orbitalPattern)" opacity="0.1">
    <animateTransform attributeName="transform" attributeType="XML" type="translate" values="0,0;3,2;0,0;-2,3;0,0" dur="12s" repeatCount="indefinite"/>
  </rect>
  
  <!-- Main wand certificate background -->
  <rect x="15" y="25" width="270" height="400" rx="20" ry="20" fill="url(#wandGradient)" stroke="url(#orbitalGold)" stroke-width="4" filter="url(#orbitalGlow)">
    <animateTransform attributeName="transform" attributeType="XML" type="scale" values="1;1.005;1" dur="8s" repeatCount="indefinite" transform-origin="150 225"/>
  </rect>
  
  <!-- Inner mystical border -->
  <rect x="25" y="35" width="250" height="380" rx="15" ry="15" fill="none" stroke="url(#cosmicGlow)" stroke-width="2" opacity="0.7">
    <animate attributeName="stroke-dasharray" values="0,1200;1200,0;0,1200" dur="15s" repeatCount="indefinite"/>
    <animate attributeName="opacity" values="0.7;1;0.7" dur="10s" repeatCount="indefinite"/>
  </rect>
  
  <!-- Orbital wand illustration -->
  <g transform="translate(150, 120)">
    <!-- Wand handle (bottom) -->
    <rect x="-8" y="40" width="16" height="80" rx="8" ry="8" fill="url(#wandCore)" stroke="url(#orbitalGold)" stroke-width="2" filter="url(#wandShimmer)"/>
    
    <!-- Handle grip details -->
    <rect x="-6" y="50" width="12" height="4" rx="2" fill="url(#orbitalGold)" opacity="0.8"/>
    <rect x="-6" y="60" width="12" height="4" rx="2" fill="url(#orbitalGold)" opacity="0.8"/>
    <rect x="-6" y="70" width="12" height="4" rx="2" fill="url(#orbitalGold)" opacity="0.8"/>
    
    <!-- Wand shaft (middle) -->
    <rect x="-3" y="-20" width="6" height="60" rx="3" ry="3" fill="url(#wandCore)" stroke="url(#orbitalGold)" stroke-width="1" filter="url(#wandShimmer)"/>
    
    <!-- Magical orb (top) -->
    <circle cx="0" cy="-35" r="18" fill="url(#magicalOrb)" stroke="url(#orbitalGold)" stroke-width="2" filter="url(#orbitalGlow)">
      <animate attributeName="r" values="18;20;18" dur="4s" repeatCount="indefinite"/>
    </circle>
    
    <!-- Inner orb core -->
    <circle cx="0" cy="-35" r="12" fill="url(#cosmicGlow)" opacity="0.8">
      <animate attributeName="opacity" values="0.8;1;0.8" dur="3s" repeatCount="indefinite"/>
    </circle>
    
    <!-- Orbital rings around the orb -->
    <circle cx="0" cy="-35" r="25" fill="none" stroke="url(#cosmicGlow)" stroke-width="1.5" opacity="0.6" stroke-dasharray="3,3">
      <animateTransform attributeName="transform" attributeType="XML" type="rotate" from="0 0 -35" to="360 0 -35" dur="8s" repeatCount="indefinite"/>
    </circle>
    <circle cx="0" cy="-35" r="30" fill="none" stroke="url(#orbitalGold)" stroke-width="1" opacity="0.4" stroke-dasharray="2,4">
      <animateTransform attributeName="transform" attributeType="XML" type="rotate" from="360 0 -35" to="0 0 -35" dur="12s" repeatCount="indefinite"/>
    </circle>
    
    <!-- Floating dust particles -->
    <g fill="url(#dustGlow)" filter="url(#dustShimmer)">
      <circle cx="-35" cy="-20" r="1.5" opacity="0.8">
        <animate attributeName="opacity" values="0.8;0.3;0.8" dur="2s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;5,-3;0,0" dur="6s" repeatCount="indefinite"/>
      </circle>
      <circle cx="35" cy="-25" r="1.2" opacity="0.6">
        <animate attributeName="opacity" values="0.6;0.2;0.6" dur="1.8s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;-4,2;0,0" dur="5s" repeatCount="indefinite"/>
      </circle>
      <circle cx="-25" cy="-45" r="1.3" opacity="0.7">
        <animate attributeName="opacity" values="0.7;0.2;0.7" dur="2.2s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;3,4;0,0" dur="7s" repeatCount="indefinite"/>
      </circle>
      <circle cx="28" cy="-50" r="1" opacity="0.9">
        <animate attributeName="opacity" values="0.9;0.3;0.9" dur="1.5s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="translate" values="0,0;-2,-3;0,0" dur="4s" repeatCount="indefinite"/>
      </circle>
    </g>
  </g>
  
  <!-- Wand title -->
  <text x="150" y="200" font-family="serif" font-size="24" font-weight="bold" text-anchor="middle" fill="url(#cosmicGlow)" filter="url(#orbitalGlow)">Orbital Wand #{}</text>
  <text x="150" y="220" font-family="serif" font-size="14" text-anchor="middle" fill="url(#orbitalGold)" opacity="0.9">{}</text>
  
  <!-- Decorative divider -->
  <g transform="translate(150, 235)">
    <line x1="-80" y1="0" x2="80" y2="0" stroke="url(#cosmicGlow)" stroke-width="2" opacity="0.6"/>
    <circle cx="-60" cy="0" r="3" fill="url(#orbitalGold)" opacity="0.8"/>
    <circle cx="-30" cy="0" r="2" fill="url(#magicalOrb)" opacity="0.8"/>
    <circle cx="0" cy="0" r="4" fill="url(#cosmicGlow)" opacity="0.8"/>
    <circle cx="30" cy="0" r="2" fill="url(#magicalOrb)" opacity="0.8"/>
    <circle cx="60" cy="0" r="3" fill="url(#orbitalGold)" opacity="0.8"/>
  </g>
  
  <!-- Wand stats -->
  <rect x="50" y="250" width="200" height="35" rx="8" ry="8" fill="url(#orbitalGold)" opacity="0.1" stroke="url(#cosmicGlow)" stroke-width="1.5"/>
  <text x="150" y="265" font-family="serif" font-size="11" font-weight="bold" text-anchor="middle" fill="url(#orbitalGold)" opacity="0.9">POWER LEVEL</text>
  <text x="150" y="280" font-family="serif" font-size="16" font-weight="bold" text-anchor="middle" fill="url(#cosmicGlow)" filter="url(#wandShimmer)">{}</text>
  
  <!-- Position token source -->
  <rect x="50" y="295" width="200" height="30" rx="6" ry="6" fill="url(#orbitalGold)" opacity="0.08" stroke="url(#wandCore)" stroke-width="1" opacity="0.4"/>
  <text x="150" y="308" font-family="serif" font-size="9" text-anchor="middle" fill="url(#wandCore)" opacity="0.8">Forged from Position Token</text>
  <text x="150" y="320" font-family="serif" font-size="11" text-anchor="middle" fill="url(#cosmicGlow)" opacity="0.9">{}</text>
  
  <!-- Dust enhancement -->
  <rect x="50" y="330" width="200" height="25" rx="6" ry="6" fill="url(#dustGlow)" opacity="0.1" stroke="url(#dustGlow)" stroke-width="1" opacity="0.6"/>
  <text x="150" y="342" font-family="serif" font-size="9" text-anchor="middle" fill="url(#dustGlow)" opacity="0.8">Dust Enhancement</text>
  <text x="150" y="352" font-family="serif" font-size="10" text-anchor="middle" fill="url(#dustGlow)" opacity="0.9">{} (+{} bonus)</text>
  
  <!-- Magical signature -->
  <text x="150" y="370" font-family="serif" font-size="8" text-anchor="middle" fill="url(#wandCore)" opacity="0.7">Tx: {}</text>
  <text x="150" y="382" font-family="serif" font-size="8" text-anchor="middle" fill="url(#wandCore)" opacity="0.7">Merkle: {}</text>
  <text x="150" y="394" font-family="serif" font-size="8" text-anchor="middle" fill="url(#wandCore)" opacity="0.7">XOR: {} + {} = {} (Victory!)</text>
  
  <!-- Floating cosmic elements -->
  <g fill="url(#cosmicGlow)" filter="url(#wandShimmer)">
    <!-- Orbiting stars -->
    <g transform="translate(80, 340)">
      <polygon points="0,-3 1,-1 3,0 1,1 0,3 -1,1 -3,0 -1,-1" fill="url(#cosmicGlow)"/>
      <animate attributeName="transform" values="translate(80,340) rotate(0);translate(80,335) rotate(180);translate(80,340) rotate(360)" dur="6s" repeatCount="indefinite"/>
    </g>
    <g transform="translate(220, 340)">
      <polygon points="0,-3 1,-1 3,0 1,1 0,3 -1,1 -3,0 -1,-1" fill="url(#cosmicGlow)"/>
      <animate attributeName="transform" values="translate(220,340) rotate(0);translate(220,345) rotate(-180);translate(220,340) rotate(-360)" dur="6s" repeatCount="indefinite"/>
    </g>
    
    <!-- Magical runes -->
    <g transform="translate(70, 410)">
      <circle cx="0" cy="0" r="5" fill="url(#orbitalGold)" opacity="0.8"/>
      <text x="0" y="2" font-family="serif" font-size="6" text-anchor="middle" fill="#0f0f23" font-weight="bold">⚡</text>
      <animate attributeName="transform" values="translate(70,410) scale(1);translate(70,405) scale(1.1);translate(70,410) scale(1)" dur="3s" repeatCount="indefinite"/>
    </g>
    <g transform="translate(230, 410)">
      <circle cx="0" cy="0" r="5" fill="url(#orbitalGold)" opacity="0.8"/>
      <text x="0" y="2" font-family="serif" font-size="6" text-anchor="middle" fill="#0f0f23" font-weight="bold">✦</text>
      <animate attributeName="transform" values="translate(230,410) scale(1);translate(230,415) scale(1.1);translate(230,410) scale(1)" dur="3s" repeatCount="indefinite"/>
    </g>
  </g>
  
  <!-- Cosmic sparkles -->
  <g fill="url(#cosmicGlow)" filter="url(#wandShimmer)">
    <circle cx="60" cy="80" r="1.5" opacity="0.8">
      <animate attributeName="opacity" values="0.8;0.3;0.8" dur="2.5s" repeatCount="indefinite"/>
    </circle>
    <circle cx="240" cy="90" r="1.2" opacity="0.6">
      <animate attributeName="opacity" values="0.6;0.2;0.6" dur="1.8s" repeatCount="indefinite"/>
    </circle>
    <circle cx="80" cy="180" r="2" opacity="0.7">
      <animate attributeName="opacity" values="0.7;0.2;0.7" dur="3s" repeatCount="indefinite"/>
    </circle>
    <circle cx="220" cy="170" r="1.4" opacity="0.5">
      <animate attributeName="opacity" values="0.5;0.1;0.5" dur="2.2s" repeatCount="indefinite"/>
    </circle>
  </g>
  
  <!-- Outer cosmic aura -->
  <rect x="12" y="22" width="276" height="406" rx="22" ry="22" fill="none" stroke="url(#cosmicGlow)" stroke-width="2" opacity="0.5">
    <animate attributeName="stroke-width" values="2;4;2" dur="6s" repeatCount="indefinite"/>
    <animate attributeName="opacity" values="0.5;0.9;0.5" dur="6s" repeatCount="indefinite"/>
  </rect>
</svg>"##, 
            magical_color,
            data.wand_id,
            wand_type,
            wand_power,
            position_display,
            dust_display,
            data.dust_bonus,
            txid_short,
            merkle_short,
            data.base_xor_result,
            data.dust_bonus,
            data.final_xor_result
        );

        Ok(svg)
    }

    fn calculate_wand_power(final_xor_result: u8) -> String {
        match final_xor_result {
            141..=160 => "Apprentice".to_string(),
            161..=180 => "Adept".to_string(),
            181..=200 => "Expert".to_string(),
            201..=220 => "Master".to_string(),
            221..=240 => "Grandmaster".to_string(),
            241..=255 => "Cosmic".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn get_wand_type(final_xor_result: u8) -> String {
        match final_xor_result % 7 {
            0 => "Stellar Wand".to_string(),
            1 => "Nebula Wand".to_string(),
            2 => "Quantum Wand".to_string(),
            3 => "Cosmic Wand".to_string(),
            4 => "Void Wand".to_string(),
            5 => "Plasma Wand".to_string(),
            6 => "Orbital Wand".to_string(),
            _ => "Mystery Wand".to_string(),
        }
    }

    fn get_magical_color(final_xor_result: u8) -> &'static str {
        match final_xor_result % 6 {
            0 => "#fbbf24", // Gold
            1 => "#34d399", // Emerald
            2 => "#60a5fa", // Blue
            3 => "#a78bfa", // Purple
            4 => "#f87171", // Red
            5 => "#fbbf24", // Gold
            _ => "#a78bfa", // Purple default
        }
    }

    fn format_dust_amount(amount: u128) -> String {
        if amount >= 1_000_000 {
            format!("{:.1}M", amount as f64 / 1_000_000.0)
        } else if amount >= 1_000 {
            format!("{:.1}K", amount as f64 / 1_000.0)
        } else {
            amount.to_string()
        }
    }
}