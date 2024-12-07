#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use half::f16;
    
    #[test]
    fn test_half() {
        let res = f16::from_f32(0.1) + f16::from_f32(0.2);

        assert_abs_diff_eq!(f32::from(res), 0.3, epsilon = 1e-3);
        assert_eq!(res.to_le_bytes(), [0xcc, 0x34]);
    }

    use ndarray::{array, s, Array};

    #[test]
    fn test_ndarray() {
        let a = array![[1, 2], [3, 4]];

        assert_eq!(a.slice(s![.., 1]), Array::from_vec(vec![2, 4]));
    }

    use nalgebra::Matrix2;

    #[test]
    fn test_naldebra() {
        let m = Matrix2::new(1., 2.,
                             3., 4.);

        assert_eq!(m.determinant(), -2.);
        assert_eq!(m.is_invertible(), true);
    }
}
