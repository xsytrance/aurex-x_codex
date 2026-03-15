use aurex_scene::{
    KeyLight, Scene, SdfMaterial, SdfMaterialType, SdfModifier, SdfNode, SdfObject, SdfPrimitive,
    Vec3,
};

pub const MAX_BOOT_OBJECTS: usize = 10;

pub fn rebuild_minimal_boot_scene(scene: &mut Scene, include_library_bar: bool) {
    scene.sdf.objects.clear();

    let sphere = SdfObject {
        primitive: SdfPrimitive::Sphere { radius: 2.0 },
        modifiers: vec![SdfModifier::Translate {
            offset: Vec3::new(0.0, 0.0, 0.0),
        }],
        material: SdfMaterial {
            material_type: SdfMaterialType::SolidColor,
            base_color: Vec3::new(0.72, 0.82, 0.96),
            emissive_strength: 0.06,
            ..SdfMaterial::default()
        },
        bounds_radius: Some(2.1),
    };
    scene.sdf.objects.push(sphere);

    if include_library_bar {
        scene.sdf.objects.push(SdfObject {
            primitive: SdfPrimitive::Box {
                size: Vec3::new(5.5, 0.25, 0.25),
            },
            modifiers: vec![SdfModifier::Translate {
                offset: Vec3::new(0.0, -3.0, 0.0),
            }],
            material: SdfMaterial {
                material_type: SdfMaterialType::SolidColor,
                base_color: Vec3::new(0.2, 0.24, 0.3),
                emissive_strength: 0.02,
                ..SdfMaterial::default()
            },
            bounds_radius: Some(5.6),
        });
    }

    if scene.sdf.objects.len() > MAX_BOOT_OBJECTS {
        scene.sdf.objects.truncate(MAX_BOOT_OBJECTS);
    }

    scene.sdf.root = SdfNode::Empty;
}

pub fn ensure_minimal_boot_lighting(scene: &mut Scene) {
    if let Some(light) = scene.sdf.lighting.key_lights.first_mut() {
        light.direction = Vec3::new(0.5, -1.0, -0.5);
        light.intensity = 3.0;
        light.color = Vec3::new(1.0, 0.98, 0.92);
    } else {
        scene.sdf.lighting.key_lights.push(KeyLight {
            direction: Vec3::new(0.5, -1.0, -0.5),
            intensity: 3.0,
            color: Vec3::new(1.0, 0.98, 0.92),
        });
    }
}
