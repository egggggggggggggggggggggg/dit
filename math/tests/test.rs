#[cfg(test)]
mod tests {
    use math::lalg::Vec2;

    #[test]
    fn negate_vector() {
        let vec = Vec2 { x: 1.0, y: 1.0 };
        let negated_vector = -vec;
        assert_eq!(negated_vector, Vec2 { x: -1.0, y: -1.0 });
    }
}
