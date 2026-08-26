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

const LOW_EXTRAPOLATION_USER_PERCENTS: (u8, u8) = (10, 20);
const HIGH_EXTRAPOLATION_USER_PERCENTS: (u8, u8) = (80, 90);

pub fn hardware_percent_for_user_percent(user_percent: u8) -> u8 {
    let user_percent = user_percent.min(100);
    if user_percent < LOW_EXTRAPOLATION_USER_PERCENTS.0 {
        return extrapolate_hardware_percent(
            LOW_EXTRAPOLATION_USER_PERCENTS.0,
            LOW_EXTRAPOLATION_USER_PERCENTS.1,
            user_percent,
        );
    }
    if user_percent > HIGH_EXTRAPOLATION_USER_PERCENTS.1 {
        return extrapolate_hardware_percent(
            HIGH_EXTRAPOLATION_USER_PERCENTS.0,
            HIGH_EXTRAPOLATION_USER_PERCENTS.1,
            user_percent,
        );
    }
    measured_hardware_percent_for_user_percent(user_percent)
}

fn extrapolate_hardware_percent(user_a: u8, user_b: u8, user_percent: u8) -> u8 {
    let hardware_a = f32::from(measured_hardware_percent_for_user_percent(user_a));
    let hardware_b = f32::from(measured_hardware_percent_for_user_percent(user_b));
    let t = (f32::from(user_percent) - f32::from(user_a)) / (f32::from(user_b) - f32::from(user_a));
    (hardware_a + t * (hardware_b - hardware_a))
        .round()
        .clamp(0.0, 100.0) as u8
}

fn measured_hardware_percent_for_user_percent(user_percent: u8) -> u8 {
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
    fn interior_ten_percent_steps_keep_measured_mapping() {
        let expected = [
            (10, 25),
            (20, 30),
            (30, 35),
            (40, 38),
            (50, 41),
            (60, 45),
            (70, 48),
            (80, 51),
            (90, 55),
        ];
        for (user_percent, hardware_percent) in expected {
            assert_eq!(
                hardware_percent_for_user_percent(user_percent),
                hardware_percent,
                "user {user_percent}%"
            );
        }
    }

    #[test]
    fn endpoints_follow_adjacent_step_extrapolation() {
        assert_eq!(hardware_percent_for_user_percent(0), 20);
        assert_eq!(hardware_percent_for_user_percent(100), 59);
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
