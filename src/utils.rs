use iced::Color;

pub const ASYNC_CHANNEL_SIZE: usize = 10;

pub trait ColorUtils {
    fn darken(self, percentage: f32) -> Self;
    fn lighten(self, percentage: f32) -> Self;
}

impl ColorUtils for Color {
    fn darken(self, percentage: f32) -> Self {
        let Color { r, g, b, a } = self;
        let factor = 1.0 - percentage;
        Color::from_rgba(
            (r * factor).clamp(0.0, 1.0),
            (g * factor).clamp(0.0, 1.0),
            (b * factor).clamp(0.0, 1.0),
            a,
        )
    }

    fn lighten(self, percentage: f32) -> Self {
        let Color { r, g, b, a } = self;
        let factor = 1.0 + percentage;
        Color::from_rgba(
            (r * factor).clamp(0.0, 1.0),
            (g * factor).clamp(0.0, 1.0),
            (b * factor).clamp(0.0, 1.0),
            a,
        )
    }
}
