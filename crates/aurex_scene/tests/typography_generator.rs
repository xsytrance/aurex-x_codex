use aurex_scene::typography_generator::TypographyGenerator;

#[test]
fn letter_generation_is_deterministic() {
    let a = TypographyGenerator::new(2026).generate_letter('A', [0.0, 0.0, 0.0]);
    let b = TypographyGenerator::new(2026).generate_letter('A', [0.0, 0.0, 0.0]);
    assert_eq!(a, b);
}

#[test]
fn word_generation_is_deterministic() {
    let generator = TypographyGenerator::new(42);
    let first = generator.generate_word("AUREX-X");
    let second = generator.generate_word("AUREX-X");

    assert_eq!(first, second);
    assert!(first.len() >= 200);
}

#[test]
fn letter_instance_count_is_correct() {
    let generator = TypographyGenerator::new(7);
    let letter = generator.generate_letter('A', [0.0, 0.0, 0.0]);

    // A mask has 14 occupied cells, each emits 4 structural instances.
    assert_eq!(letter.len(), 56);
}
