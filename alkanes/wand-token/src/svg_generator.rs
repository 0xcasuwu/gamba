use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WandData {
    pub wand_id: u128,
    pub dust_amount: u128,
    pub base_xor: u8,
    pub dust_bonus: u8,
    pub final_result: u8,
    pub creation_block: u128,
    pub current_block: u128,
    pub wand_type: String,
    pub wand_power: String,
}

pub struct SvgGenerator;

impl SvgGenerator {
    /// Generate an SVG representation of a wand based on its properties
    pub fn generate_svg(data: WandData) -> Result<String> {
        let WandData {
            wand_id,
            dust_amount,
            base_xor,
            dust_bonus,
            final_result,
            creation_block,
            current_block,
            wand_type,
            wand_power,
        } = data;

        // Calculate colors based on wand properties
        let (primary_color, secondary_color, gem_color) = Self::calculate_colors(final_result, base_xor);
        let staff_width = Self::calculate_staff_width(&wand_type);
        let gem_size = Self::calculate_gem_size(final_result);
        let sparkle_count = Self::calculate_sparkle_count(dust_amount);

        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600" width="400" height="600">
  <defs>
    <linearGradient id="staffGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:{primary_color};stop-opacity:1" />
      <stop offset="100%" style="stop-color:{secondary_color};stop-opacity:1" />
    </linearGradient>
    <radialGradient id="gemGradient" cx="50%" cy="30%" r="50%">
      <stop offset="0%" style="stop-color:{gem_color};stop-opacity:0.9" />
      <stop offset="70%" style="stop-color:{gem_color};stop-opacity:0.7" />
      <stop offset="100%" style="stop-color:{black_color};stop-opacity:0.3" />
    </radialGradient>
    <filter id="glow">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
  
  <!-- Background -->
  <rect width="400" height="600" fill="{bg_color}"/>
  
  <!-- Mystical background pattern -->
  <g opacity="0.1">
    {background_pattern}
  </g>
  
  <!-- Wand Staff -->
  <rect x="{staff_x}" y="150" width="{staff_width}" height="350"
        fill="url(#staffGradient)" rx="5" filter="url(#glow)"/>
  
  <!-- Staff decorations -->
  {staff_decorations}
  
  <!-- Wand Head/Gem -->
  <circle cx="200" cy="120" r="{gem_size}"
          fill="url(#gemGradient)" filter="url(#glow)"/>
  
  <!-- Inner gem detail -->
  <circle cx="195" cy="115" r="{inner_gem_size}"
          fill="{gem_color}" opacity="0.8"/>
  
  <!-- Magical sparkles -->
  {sparkles}
  
  <!-- Wand tip glow -->
  <circle cx="200" cy="120" r="{glow_size}"
          fill="{gem_color}" opacity="0.3" filter="url(#glow)"/>
  
  <!-- Title -->
  <text x="200" y="50" text-anchor="middle" font-family="serif" font-size="24"
        fill="{primary_color}" font-weight="bold">{wand_type} Wand</text>
  
  <!-- Wand ID -->
  <text x="200" y="75" text-anchor="middle" font-family="monospace" font-size="16"
        fill="{gray_color}">#{wand_id}</text>
  
  <!-- Stats Panel -->
  <rect x="50" y="520" width="300" height="70" fill="rgba(0,0,0,0.7)"
        stroke="{primary_color}" stroke-width="1" rx="5"/>
  
  <!-- Stats Text -->
  <text x="70" y="540" font-family="monospace" font-size="12" fill="{light_gray_color}">
    Power: {final_result} | DUST: {dust_amount}
  </text>
  <text x="70" y="555" font-family="monospace" font-size="12" fill="{light_gray_color}">
    XOR: {base_xor} | Bonus: +{dust_bonus}
  </text>
  <text x="70" y="570" font-family="monospace" font-size="10" fill="{gray_color}">
    Block: {creation_block}
  </text>
</svg>"#,
            primary_color = primary_color,
            secondary_color = secondary_color,
            gem_color = gem_color,
            black_color = "#000000",
            bg_color = "#0a0a0a",
            gray_color = "#888888",
            light_gray_color = "#cccccc",
            staff_x = 200 - staff_width / 2,
            staff_width = staff_width,
            gem_size = gem_size,
            inner_gem_size = gem_size - 5,
            glow_size = gem_size + 10,
            sparkles = Self::generate_sparkles(sparkle_count, &gem_color),
            staff_decorations = Self::generate_staff_decorations(staff_width, &primary_color),
            background_pattern = Self::generate_background_pattern(&wand_type),
            wand_type = wand_type,
            wand_id = wand_id,
            final_result = final_result,
            dust_amount = dust_amount,
            base_xor = base_xor,
            dust_bonus = dust_bonus,
            creation_block = creation_block,
        );

        Ok(svg)
    }

    /// Calculate colors based on wand properties
    fn calculate_colors(final_result: u8, base_xor: u8) -> (String, String, String) {
        let primary = match final_result {
            250..=255 => "#ff6b35", // Legendary - Orange/Red
            230..=249 => "#a855f7", // Epic - Purple
            200..=229 => "#3b82f6", // Rare - Blue
            170..=199 => "#10b981", // Uncommon - Green
            145..=169 => "#6b7280", // Common - Gray
            _ => "#374151", // Failed - Dark Gray
        };

        let secondary = match final_result {
            250..=255 => "#dc2626",
            230..=249 => "#7c3aed",
            200..=229 => "#1d4ed8",
            170..=199 => "#059669",
            145..=169 => "#4b5563",
            _ => "#1f2937",
        };

        // Gem color varies based on XOR value for uniqueness
        let gem = match base_xor % 6 {
            0 => "#ef4444", // Red
            1 => "#3b82f6", // Blue
            2 => "#10b981", // Green
            3 => "#f59e0b", // Yellow
            4 => "#a855f7", // Purple
            _ => "#06b6d4", // Cyan
        };

        (primary.to_string(), secondary.to_string(), gem.to_string())
    }

    /// Calculate staff width based on wand type
    fn calculate_staff_width(wand_type: &str) -> u32 {
        match wand_type {
            "Legendary" => 16,
            "Epic" => 14,
            "Rare" => 12,
            "Uncommon" => 10,
            "Common" => 8,
            _ => 6,
        }
    }

    /// Calculate gem size based on final result
    fn calculate_gem_size(final_result: u8) -> u32 {
        match final_result {
            250..=255 => 25,
            230..=249 => 22,
            200..=229 => 20,
            170..=199 => 18,
            145..=169 => 15,
            _ => 12,
        }
    }

    /// Calculate number of sparkles based on DUST amount
    fn calculate_sparkle_count(dust_amount: u128) -> u32 {
        match dust_amount {
            10000.. => 15,
            5000..=9999 => 12,
            1000..=4999 => 8,
            100..=999 => 5,
            _ => 3,
        }
    }

    /// Generate sparkle effects around the wand
    fn generate_sparkles(count: u32, gem_color: &str) -> String {
        let mut sparkles = String::new();
        
        for i in 0..count {
            let angle = (i as f32 * 360.0 / count as f32).to_radians();
            let radius = 80.0 + (i as f32 * 10.0) % 40.0;
            let x = 200.0 + radius * angle.cos();
            let y = 200.0 + radius * angle.sin();
            let size = 2 + (i % 3);
            let opacity = 0.6 + (i as f32 * 0.1) % 0.4;
            
            sparkles.push_str(&format!(
                r#"<circle cx="{:.1}" cy="{:.1}" r="{}" fill="{}" opacity="{:.2}">
    <animate attributeName="opacity" values="{:.2};0.2;{:.2}" dur="2s" repeatCount="indefinite"/>
  </circle>"#,
                x, y, size, gem_color, opacity, opacity, opacity
            ));
        }
        
        sparkles
    }

    /// Generate decorative elements on the staff
    fn generate_staff_decorations(staff_width: u32, primary_color: &str) -> String {
        let staff_center = 200;
        let decoration_spacing = 60;
        let mut decorations = String::new();
        
        for i in 0..4 {
            let y = 180 + i * decoration_spacing;
            let decoration_width = staff_width + 6;
            
            decorations.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="3" fill="{}" opacity="0.7"/>"#,
                staff_center - decoration_width / 2,
                y,
                decoration_width,
                primary_color
            ));
        }
        
        decorations
    }

    /// Generate background pattern based on wand type
    fn generate_background_pattern(wand_type: &str) -> String {
        match wand_type {
            "Legendary" => {
                format!(r#"<circle cx="100" cy="150" r="30" fill="{}"/>
                   <circle cx="300" cy="250" r="25" fill="{}"/>
                   <circle cx="80" cy="400" r="20" fill="{}"/>
                   <circle cx="320" cy="450" r="35" fill="{}"/>"#,
                   "#ff6b35", "#dc2626", "#f97316", "#ea580c")
            }
            "Epic" => {
                format!(r#"<polygon points="100,100 120,140 80,140" fill="{}"/>
                   <polygon points="300,200 320,240 280,240" fill="{}"/>
                   <polygon points="150,450 170,490 130,490" fill="{}"/>"#,
                   "#a855f7", "#7c3aed", "#8b5cf6")
            }
            "Rare" => {
                format!(r#"<rect x="50" y="200" width="20" height="20" fill="{}" transform="rotate(45 60 210)"/>
                   <rect x="330" y="300" width="15" height="15" fill="{}" transform="rotate(45 337 307)"/>
                   <rect x="100" y="500" width="25" height="25" fill="{}" transform="rotate(45 112 512)"/>"#,
                   "#3b82f6", "#1d4ed8", "#2563eb")
            }
            _ => {
                format!(r#"<circle cx="120" cy="200" r="10" fill="{}"/>
                   <circle cx="280" cy="350" r="8" fill="{}"/>
                   <circle cx="150" cy="480" r="12" fill="{}"/>"#,
                   "#6b7280", "#4b5563", "#374151")
            }
        }
    }

    /// Generate JSON attributes for the wand token
    pub fn get_attributes(data: WandData) -> Result<String> {
        let WandData {
            wand_id,
            dust_amount,
            base_xor,
            dust_bonus,
            final_result,
            creation_block,
            current_block,
            wand_type,
            wand_power,
        } = data;

        let age = current_block.saturating_sub(creation_block);
        let rarity_score = Self::calculate_rarity_score(final_result, dust_amount);

        let attributes = format!(
            r#"{{
  "name": "{} Wand #{}",
  "description": "A mystical wand forged from blockchain entropy and DUST tokens. Each wand is unique, with power determined by cryptographic randomness.",
  "image": "data:image/svg+xml;base64,{{SVG_DATA}}",
  "attributes": [
    {{
      "trait_type": "Type",
      "value": "{}"
    }},
    {{
      "trait_type": "Power Level",
      "value": {}
    }},
    {{
      "trait_type": "DUST Amount",
      "value": {}
    }},
    {{
      "trait_type": "Base XOR",
      "value": {}
    }},
    {{
      "trait_type": "DUST Bonus",
      "value": {}
    }},
    {{
      "trait_type": "Final Result",
      "value": {}
    }},
    {{
      "trait_type": "Creation Block",
      "value": {}
    }},
    {{
      "trait_type": "Age (Blocks)",
      "value": {}
    }},
    {{
      "trait_type": "Rarity Score",
      "value": {}
    }},
    {{
      "trait_type": "Magical Power",
      "value": "{}"
    }}
  ]
}}"#,
            wand_type,
            wand_id,
            wand_type,
            final_result,
            dust_amount,
            base_xor,
            dust_bonus,
            final_result,
            creation_block,
            age,
            rarity_score,
            wand_power
        );

        Ok(attributes)
    }

    /// Calculate a rarity score based on various factors
    fn calculate_rarity_score(final_result: u8, dust_amount: u128) -> u32 {
        let mut score = final_result as u32;
        
        // Add bonus for DUST amount
        if dust_amount >= 10000 {
            score += 50;
        } else if dust_amount >= 5000 {
            score += 30;
        } else if dust_amount >= 1000 {
            score += 15;
        } else if dust_amount >= 100 {
            score += 5;
        }
        
        // Add bonus for high power levels
        match final_result {
            250..=255 => score += 100, // Legendary bonus
            230..=249 => score += 50,  // Epic bonus
            200..=229 => score += 25,  // Rare bonus
            170..=199 => score += 10,  // Uncommon bonus
            _ => {}
        }
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_generation() {
        let data = WandData {
            wand_id: 1,
            dust_amount: 5000,
            base_xor: 200,
            dust_bonus: 25,
            final_result: 225,
            creation_block: 1000,
            current_block: 1100,
            wand_type: "Rare".to_string(),
            wand_power: "Greater Enchantment (DUST-Enhanced)".to_string(),
        };

        let svg = SvgGenerator::generate_svg(data).unwrap();
        assert!(svg.contains("svg"));
        assert!(svg.contains("Rare Wand"));
        assert!(svg.contains("#1"));
    }

    #[test]
    fn test_attributes_generation() {
        let data = WandData {
            wand_id: 1,
            dust_amount: 5000,
            base_xor: 200,
            dust_bonus: 25,
            final_result: 225,
            creation_block: 1000,
            current_block: 1100,
            wand_type: "Rare".to_string(),
            wand_power: "Greater Enchantment (DUST-Enhanced)".to_string(),
        };

        let attributes = SvgGenerator::get_attributes(data).unwrap();
        assert!(attributes.contains("Rare Wand #1"));
        assert!(attributes.contains("\"value\": 225"));
        assert!(attributes.contains("\"value\": 5000"));
    }

    #[test]
    fn test_color_calculation() {
        let (primary, secondary, gem) = SvgGenerator::calculate_colors(255, 100);
        assert_eq!(primary, "#ff6b35"); // Legendary color
        
        let (primary, secondary, gem) = SvgGenerator::calculate_colors(150, 100);
        assert_eq!(primary, "#6b7280"); // Common color
    }

    #[test]
    fn test_rarity_score() {
        let score = SvgGenerator::calculate_rarity_score(255, 10000);
        assert!(score > 300); // Should be high for legendary with lots of DUST
        
        let score = SvgGenerator::calculate_rarity_score(150, 0);
        assert!(score < 200); // Should be lower for common with no DUST
    }
}