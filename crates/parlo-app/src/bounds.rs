#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub width: u32,
    pub height: u32,
}

pub const MAIN: Limits = Limits {
    width: 380,
    height: 460,
};

pub const CAPTIONS: Limits = Limits {
    width: 320,
    height: 140,
};

pub const LIVE: Limits = Limits {
    width: 480,
    height: 260,
};

pub fn limits(label: &str) -> Option<Limits> {
    match label {
        "main" => Some(MAIN),
        "captions" => Some(CAPTIONS),
        "live" => Some(LIVE),
        _ => None,
    }
}

pub fn clamp(width: u32, height: u32, limits: Limits) -> (u32, u32) {
    (width.max(limits.width), height.max(limits.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_horizontal_drag_cannot_go_below_the_width() {
        assert_eq!(clamp(120, 700, MAIN), (380, 700));
    }

    #[test]
    fn a_vertical_drag_cannot_go_below_the_height() {
        assert_eq!(clamp(900, 90, MAIN), (900, 460));
    }

    #[test]
    fn a_corner_drag_cannot_go_below_either_axis() {
        assert_eq!(clamp(120, 90, MAIN), (380, 460));
    }

    #[test]
    fn a_size_above_both_minimums_is_left_alone() {
        assert_eq!(clamp(1180, 820, MAIN), (1180, 820));
        assert_eq!(clamp(720, 240, CAPTIONS), (720, 240));
        assert_eq!(clamp(860, 460, LIVE), (860, 460));
    }

    #[test]
    fn every_resizable_window_has_limits() {
        assert_eq!(limits("main"), Some(MAIN));
        assert_eq!(limits("captions"), Some(CAPTIONS));
        assert_eq!(limits("live"), Some(LIVE));
        assert_eq!(limits("island"), None);
    }

    #[test]
    fn the_native_configuration_carries_the_same_minimums() {
        let config = include_str!("../tauri.conf.json");
        for (label, limits) in [("main", MAIN), ("captions", CAPTIONS), ("live", LIVE)] {
            let start = config.find(&format!("\"label\": \"{label}\""));
            let block = &config[start.expect("a window block")..];
            let width = format!("\"minWidth\": {}", limits.width);
            let height = format!("\"minHeight\": {}", limits.height);
            assert!(block.contains(&width), "{label} lost {width}");
            assert!(block.contains(&height), "{label} lost {height}");
        }
    }
}
