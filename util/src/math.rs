pub fn unbalanced_wrapping_add(n1: u8, n2: usize) -> u8 {
    // Adding 256 is effectively a nop, since it will wrap around to the
    // original value. Mod out 256 to get a value between 0 and 255.
    let wrapped_n2 = (n2 % 256) as u8;
    n1.wrapping_add(wrapped_n2)
}

pub fn unbalanced_wrapping_sub(n1: u8, n2: usize) -> u8 {
    // Subtracting 256 is effectively a nop, since it will wrap around to the
    // original value. Mod out 256 to get a value between 0 and 255.
    let wrapped_n2 = (n2 % 256) as u8;
    n1.wrapping_sub(wrapped_n2)
}

#[cfg(test)]
mod tests {
    use super::{unbalanced_wrapping_add, unbalanced_wrapping_sub};

    #[test]
    fn unbalanced_wrapping_add_test() {
        assert_eq!(unbalanced_wrapping_add(0, 2), 2);
        assert_eq!(unbalanced_wrapping_add(0, 255), 255);
        assert_eq!(unbalanced_wrapping_add(0, 256), 0);
        assert_eq!(unbalanced_wrapping_add(0, 257), 1);
        assert_eq!(unbalanced_wrapping_add(0, 510), 254);
        assert_eq!(unbalanced_wrapping_add(0, 511), 255);
        assert_eq!(unbalanced_wrapping_add(0, 512), 0);

        assert_eq!(unbalanced_wrapping_add(15, 2), 17);
        assert_eq!(unbalanced_wrapping_add(15, 240), 255);
        assert_eq!(unbalanced_wrapping_add(15, 241), 0);
        assert_eq!(unbalanced_wrapping_add(15, 255), 14);
        assert_eq!(unbalanced_wrapping_add(15, 256), 15);
        assert_eq!(unbalanced_wrapping_add(15, 257), 16);
        assert_eq!(unbalanced_wrapping_add(15, 496), 255);
        assert_eq!(unbalanced_wrapping_add(15, 497), 0);
        assert_eq!(unbalanced_wrapping_add(15, 510), 13);
        assert_eq!(unbalanced_wrapping_add(15, 511), 14);
        assert_eq!(unbalanced_wrapping_add(15, 512), 15);

        assert_eq!(unbalanced_wrapping_add(255, 2), 1);
        assert_eq!(unbalanced_wrapping_add(255, 255), 254);
        assert_eq!(unbalanced_wrapping_add(255, 256), 255);
        assert_eq!(unbalanced_wrapping_add(255, 257), 0);
        assert_eq!(unbalanced_wrapping_add(255, 510), 253);
        assert_eq!(unbalanced_wrapping_add(255, 511), 254);
        assert_eq!(unbalanced_wrapping_add(255, 512), 255);
        assert_eq!(unbalanced_wrapping_add(255, 513), 0);
    }

    #[test]
    fn unbalanced_wrapping_sub_test() {
        assert_eq!(unbalanced_wrapping_sub(0, 2), 254);
        assert_eq!(unbalanced_wrapping_sub(0, 255), 1);
        assert_eq!(unbalanced_wrapping_sub(0, 256), 0);
        assert_eq!(unbalanced_wrapping_sub(0, 257), 255);
        assert_eq!(unbalanced_wrapping_sub(0, 510), 2);
        assert_eq!(unbalanced_wrapping_sub(0, 511), 1);
        assert_eq!(unbalanced_wrapping_sub(0, 512), 0);
        assert_eq!(unbalanced_wrapping_sub(0, 513), 255);

        assert_eq!(unbalanced_wrapping_sub(15, 2), 13);
        assert_eq!(unbalanced_wrapping_sub(15, 255), 16);
        assert_eq!(unbalanced_wrapping_sub(15, 256), 15);
        assert_eq!(unbalanced_wrapping_sub(15, 257), 14);
        assert_eq!(unbalanced_wrapping_sub(15, 271), 0);
        assert_eq!(unbalanced_wrapping_sub(15, 272), 255);
        assert_eq!(unbalanced_wrapping_sub(15, 510), 17);
        assert_eq!(unbalanced_wrapping_sub(15, 511), 16);
        assert_eq!(unbalanced_wrapping_sub(15, 512), 15);
        assert_eq!(unbalanced_wrapping_sub(15, 527), 0);
        assert_eq!(unbalanced_wrapping_sub(15, 528), 255);

        assert_eq!(unbalanced_wrapping_sub(255, 2), 253);
        assert_eq!(unbalanced_wrapping_sub(255, 255), 0);
        assert_eq!(unbalanced_wrapping_sub(255, 256), 255);
        assert_eq!(unbalanced_wrapping_sub(255, 257), 254);
        assert_eq!(unbalanced_wrapping_sub(255, 510), 1);
        assert_eq!(unbalanced_wrapping_sub(255, 511), 0);
        assert_eq!(unbalanced_wrapping_sub(255, 512), 255);
        assert_eq!(unbalanced_wrapping_sub(255, 513), 254);
    }
}
