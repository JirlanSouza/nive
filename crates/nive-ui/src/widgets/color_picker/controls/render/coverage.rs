use iced::{Point, Size};

pub(super) fn circle_coverage(
    center: Point,
    radius: f32,
    top_left: Point,
    pixel_size: Size,
    samples_per_axis: usize,
) -> f32 {
    coverage(top_left, pixel_size, samples_per_axis, |sample| {
        circle_contains(center, radius, sample)
    })
}

pub(super) fn rounded_rect_coverage(
    size: Size,
    top_left: Point,
    pixel_size: Size,
    radius: f32,
    samples_per_axis: usize,
) -> f32 {
    let radius = radius.min(size.width / 2.0).min(size.height / 2.0);

    coverage(top_left, pixel_size, samples_per_axis, |sample| {
        rounded_rect_contains(size, sample, radius)
    })
}

fn coverage(
    top_left: Point,
    pixel_size: Size,
    samples_per_axis: usize,
    contains: impl Fn(Point) -> bool,
) -> f32 {
    if samples_per_axis == 0 {
        return 0.0;
    }

    let mut covered = 0;
    let samples = samples_per_axis * samples_per_axis;

    for row in 0..samples_per_axis {
        for col in 0..samples_per_axis {
            let sample = Point::new(
                top_left.x + pixel_size.width * (col as f32 + 0.5) / samples_per_axis as f32,
                top_left.y + pixel_size.height * (row as f32 + 0.5) / samples_per_axis as f32,
            );

            if contains(sample) {
                covered += 1;
            }
        }
    }

    covered as f32 / samples as f32
}

fn circle_contains(center: Point, radius: f32, point: Point) -> bool {
    let dx = point.x - center.x;
    let dy = point.y - center.y;

    dx * dx + dy * dy <= radius * radius
}

fn rounded_rect_contains(size: Size, point: Point, radius: f32) -> bool {
    let inner_x = point.x.clamp(radius, size.width - radius);
    let inner_y = point.y.clamp(radius, size.height - radius);
    let dx = point.x - inner_x;
    let dy = point.y - inner_y;

    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn circle_coverage_antialiases_edge_pixels() {
        let center = Point::new(6.0, 6.0);
        let coverage = circle_coverage(center, 6.0, Point::new(6.0, 0.0), Size::UNIT, 8);

        assert!(coverage > 0.0);
        assert!(coverage < 1.0);
    }

    #[test]
    fn circle_coverage_keeps_corner_transparent() {
        let center = Point::new(6.0, 6.0);
        let coverage = circle_coverage(center, 6.0, Point::ORIGIN, Size::UNIT, 8);

        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn rounded_rect_coverage_keeps_corner_transparent() {
        let coverage =
            rounded_rect_coverage(Size::new(12.0, 136.0), Point::ORIGIN, Size::UNIT, 6.0, 4);

        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn rounded_rect_coverage_keeps_top_center_opaque() {
        let coverage = rounded_rect_coverage(
            Size::new(12.0, 136.0),
            Point::new(6.0, 0.0),
            Size::UNIT,
            6.0,
            4,
        );

        assert_eq!(coverage, 1.0);
    }
}
