use crate::{
    Scene, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject, SdfPrimitive, Vec3,
};

const LETTER_HEIGHT: usize = 5;
const CELL_SIZE: f32 = 1.0;
const LETTER_ADVANCE: f32 = 6.5;
const INSTANCES_PER_CELL: usize = 4;

const A_MASK: [&str; LETTER_HEIGHT] = ["01110", "10001", "11111", "10001", "10001"];
const U_MASK: [&str; LETTER_HEIGHT] = ["10001", "10001", "10001", "10001", "01110"];
const R_MASK: [&str; LETTER_HEIGHT] = ["11110", "10001", "11110", "10100", "10010"];
const E_MASK: [&str; LETTER_HEIGHT] = ["11111", "10000", "11110", "10000", "11111"];
const X_MASK: [&str; LETTER_HEIGHT] = ["10001", "01010", "00100", "01010", "10001"];
const DASH_MASK: [&str; LETTER_HEIGHT] = ["00000", "00000", "11111", "00000", "00000"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LetterInstance {
    pub position: [f32; 3],
    pub scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TypographyGenerator {
    seed: u64,
}

impl TypographyGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate_letter(&self, letter: char, offset: [f32; 3]) -> Vec<LetterInstance> {
        let Some(mask) = letter_mask(letter) else {
            return Vec::new();
        };

        let width = mask[0].len() as f32;
        let center_x = (width - 1.0) * 0.5;
        let center_y = (LETTER_HEIGHT as f32 - 1.0) * 0.5;

        let mut instances = Vec::new();
        for (row, line) in mask.iter().enumerate() {
            for (col, cell) in line.chars().enumerate() {
                if cell != '1' {
                    continue;
                }

                for sub_idx in 0..INSTANCES_PER_CELL {
                    let ordinal = ((row * 17 + col * 31 + sub_idx * 13) as u64) ^ self.seed;
                    let jitter_x = sample_symmetric(self.seed, ordinal.wrapping_mul(3), 0.16);
                    let jitter_y = sample_symmetric(self.seed, ordinal.wrapping_mul(5), 0.08);
                    let jitter_z = sample_symmetric(self.seed, ordinal.wrapping_mul(7), 0.22);
                    let scale = 0.24 + sample_unit(self.seed, ordinal.wrapping_mul(11)) * 0.22;

                    let sub_x = ((sub_idx % 2) as f32 - 0.5) * 0.24;
                    let sub_y = ((sub_idx / 2) as f32 - 0.5) * 0.24;

                    let x = offset[0] + (col as f32 - center_x) * CELL_SIZE + sub_x + jitter_x;
                    let y = offset[1] + (center_y - row as f32) * CELL_SIZE + sub_y + jitter_y;
                    let z = offset[2] + jitter_z;

                    instances.push(LetterInstance {
                        position: [x, y, z],
                        scale,
                    });
                }
            }
        }

        instances
    }

    pub fn generate_word(&self, word: &str) -> Vec<LetterInstance> {
        let chars: Vec<char> = word.chars().collect();
        let total_width = chars.len().saturating_sub(1) as f32 * LETTER_ADVANCE;

        let mut all = Vec::new();
        for (idx, ch) in chars.iter().enumerate() {
            let x = idx as f32 * LETTER_ADVANCE - total_width * 0.5;
            let mut letter = self.generate_letter(*ch, [x, 3.0, 0.0]);
            all.append(&mut letter);
        }

        all
    }

    pub fn apply_word_to_scene(&self, scene: &mut Scene, word: &str) {
        let instances = self.generate_word(word);
        let mut glyph_nodes = Vec::with_capacity(instances.len());

        for (idx, instance) in instances.iter().enumerate() {
            let primitive = match idx % 4 {
                0 => SdfPrimitive::Box {
                    size: Vec3::new(instance.scale, instance.scale, instance.scale),
                },
                1 => SdfPrimitive::Cylinder {
                    radius: instance.scale * 0.45,
                    half_height: instance.scale,
                },
                2 => SdfPrimitive::Sphere {
                    radius: instance.scale * 0.55,
                },
                _ => SdfPrimitive::Sphere {
                    radius: instance.scale * 0.28,
                },
            };

            glyph_nodes.push(SdfNode::Transform {
                modifiers: vec![SdfModifier::Translate {
                    offset: Vec3::new(
                        instance.position[0],
                        instance.position[1],
                        instance.position[2],
                    ),
                }],
                child: Box::new(SdfNode::Primitive {
                    object: SdfObject {
                        primitive,
                        modifiers: vec![],
                        material: SdfMaterial {
                            material_type: SdfMaterialType::SolidColor,
                            base_color: Vec3::new(0.85, 0.85, 0.92),
                            emissive_strength: 0.22,
                            ..SdfMaterial::default()
                        },
                        bounds_radius: Some(instance.scale * 2.5),
                    },
                }),
                bounds_radius: Some(instance.scale * 2.5),
            });
        }

        match &mut scene.sdf.root {
            SdfNode::Union { children } => children.extend(glyph_nodes),
            root => {
                let existing = std::mem::replace(root, SdfNode::Empty);
                *root = SdfNode::Union {
                    children: std::iter::once(existing)
                        .chain(glyph_nodes)
                        .collect::<Vec<_>>(),
                };
            }
        }
    }
}

fn letter_mask(letter: char) -> Option<&'static [&'static str; LETTER_HEIGHT]> {
    match letter.to_ascii_uppercase() {
        'A' => Some(&A_MASK),
        'U' => Some(&U_MASK),
        'R' => Some(&R_MASK),
        'E' => Some(&E_MASK),
        'X' => Some(&X_MASK),
        '-' => Some(&DASH_MASK),
        _ => None,
    }
}

fn sample_unit(seed: u64, salt: u64) -> f32 {
    let mixed = splitmix64(seed ^ salt);
    let mantissa = (mixed >> 40) as u32;
    mantissa as f32 / (u32::MAX >> 8) as f32
}

fn sample_symmetric(seed: u64, salt: u64, amplitude: f32) -> f32 {
    (sample_unit(seed, salt) * 2.0 - 1.0) * amplitude
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

pub fn apply_word_to_scene(scene: &mut Scene, word: &str) {
    TypographyGenerator::new(u64::from(scene.sdf.seed)).apply_word_to_scene(scene, word);
}
