/// Key-interior mean camera luma (YUYV Y, 0-255) measured on Original V2
/// with a locked Razer Kiyo (manual exposure 5, gain 0, auto WB/AE/AF off).
/// HID brightness is not linear: nearly all luminance change is between
/// about 25% and 55%; 60-100% is a plateau.
const MEASURED_KEY_LUMA_BY_HARDWARE_PERCENT: &[(u8, f32)] = &[
    (0, 18.37),
    (10, 19.87),
    (15, 22.39),
    (20, 26.12),
    (25, 32.03),
    (30, 43.37),
    (35, 56.43),
    (40, 81.30),
    (45, 96.30),
    (50, 119.07),
    (55, 137.68),
    (60, 139.55),
    (65, 140.95),
    (70, 142.35),
    (75, 143.75),
    (80, 145.19),
    (85, 146.61),
    (90, 148.02),
    (95, 149.45),
    (100, 149.87),
];

pub fn hardware_percent_for_user_percent(user_percent: u8) -> u8 {
    let user_percent = user_percent.min(100);
    if user_percent == 0 {
        return 0;
    }
    if user_percent == 100 {
        return 100;
    }

    let first_luma = MEASURED_KEY_LUMA_BY_HARDWARE_PERCENT[0].1;
    let last_luma = MEASURED_KEY_LUMA_BY_HARDWARE_PERCENT
        [MEASURED_KEY_LUMA_BY_HARDWARE_PERCENT.len() - 1]
        .1;
    let target_luma =
        first_luma + (last_luma - first_luma) * f32::from(user_percent) / 100.0;

    for window in MEASURED_KEY_LUMA_BY_HARDWARE_PERCENT.windows(2) {
        let (hardware_low, luma_low) = window[0];
        let (hardware_high, luma_high) = window[1];
        if target_luma > luma_high {
            continue;
        }
        if luma_high <= luma_low {
            return hardware_high;
        }
        let span = luma_high - luma_low;
        let t = (target_luma - luma_low) / span;
        let hardware = f32::from(hardware_low)
            + t * f32::from(hardware_high.saturating_sub(hardware_low));
        return hardware.round().clamp(0.0, 100.0) as u8;
    }

    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_map_to_endpoints() {
        assert_eq!(hardware_percent_for_user_percent(0), 0);
        assert_eq!(hardware_percent_for_user_percent(100), 100);
    }

    #[test]
    fn mid_range_uses_the_steep_measured_region() {
        let hardware_50 = hardware_percent_for_user_percent(50);
        let hardware_70 = hardware_percent_for_user_percent(70);
        assert!(hardware_50 >= 38 && hardware_50 <= 45, "got {hardware_50}");
        assert!(hardware_70 >= 45 && hardware_70 <= 52, "got {hardware_70}");
        assert!(hardware_70 > hardware_50);
    }

    #[test]
    fn user_steps_stay_monotonic() {
        let mapped: Vec<u8> = (0..=100)
            .step_by(10)
            .map(hardware_percent_for_user_percent)
            .collect();
        for pair in mapped.windows(2) {
            assert!(pair[1] >= pair[0], "non-monotonic {mapped:?}");
        }
    }
}
