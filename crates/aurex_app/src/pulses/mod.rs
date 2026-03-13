pub mod ambient_dreamscape;
pub mod electronic_megacity;
pub mod jazz_atmosphere;

pub use crate::pulse_builder::ExamplePulseConfig;

#[cfg(test)]
mod tests {
    use super::{
        ambient_dreamscape::create_ambient_dreamscape_pulse,
        electronic_megacity::create_electronic_megacity_pulse,
        jazz_atmosphere::create_jazz_atmosphere_pulse,
    };

    #[test]
    fn pulse_initialization_is_deterministic_for_same_seed() {
        let ea = create_electronic_megacity_pulse(77);
        let eb = create_electronic_megacity_pulse(77);
        assert_eq!(ea, eb);

        let ja = create_jazz_atmosphere_pulse(77);
        let jb = create_jazz_atmosphere_pulse(77);
        assert_eq!(ja, jb);

        let aa = create_ambient_dreamscape_pulse(77);
        let ab = create_ambient_dreamscape_pulse(77);
        assert_eq!(aa, ab);
    }
}
