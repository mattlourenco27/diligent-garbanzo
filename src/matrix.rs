use num_traits::{ConstZero, One, Zero};

#[derive(Clone, Debug, PartialEq)]
pub struct StaticMatrix<T, const ROWS: usize, const COLS: usize>(pub [[T; COLS]; ROWS]);

pub type Matrix3x3<T> = StaticMatrix<T, 3, 3>;

impl<T, const ROWS: usize, const COLS: usize> StaticMatrix<T, ROWS, COLS> {
    fn num_rows(&self) -> usize {
        ROWS
    }

    fn num_cols(&self) -> usize {
        COLS
    }

    fn dim(&self) -> (usize, usize) {
        (self.num_rows(), self.num_cols())
    }
}

impl<T, const SIZE: usize> StaticMatrix<T, SIZE, SIZE> {
    fn identity() -> Self
    where
        T: ConstZero + Copy + One + PartialEq,
    {
        let mut ret = Self::ZERO;
        for (i, row) in ret.0.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                if i == j {
                    *item = T::one();
                }
            }
        }

        ret
    }
}

impl<T, const ROWS: usize, const COLS: usize> ConstZero for StaticMatrix<T, ROWS, COLS>
where
    T: ConstZero + Copy + PartialEq,
{
    const ZERO: Self = StaticMatrix([[T::ZERO; COLS]; ROWS]);
}

impl<T, const ROWS: usize, const COLS: usize> Zero for StaticMatrix<T, ROWS, COLS>
where
    T: ConstZero + Copy + PartialEq,
{
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    fn set_zero(&mut self) {
        *self = Self::ZERO
    }

    fn zero() -> Self {
        Self::ZERO
    }
}

impl<T, const ROWS: usize, const COLS: usize> core::ops::Add for StaticMatrix<T, ROWS, COLS>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for (l_row, r_row) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            for (l, r) in l_row.iter_mut().zip(r_row.into_iter()) {
                *l = *l + r;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
   use super::StaticMatrix;

    #[test]
    fn matrix_dimensions() {
        let matrix = StaticMatrix([[1, 2], [3, 4]]);
        assert_eq!(2, matrix.num_rows());
        assert_eq!(2, matrix.num_cols());
        assert_eq!((2, 2), matrix.dim());
    }

    #[test]
    fn matrix_identity() {
        let identity3x3 = StaticMatrix::identity();
        assert_eq!(StaticMatrix([[1, 0, 0], [0, 1, 0], [0, 0, 1]]), identity3x3);
    }
}
