use num_traits::{ConstZero, One, Zero};

use crate::vector::StaticVector;

#[derive(Clone, Debug, PartialEq)]
pub struct StaticMatrix<T, const ROWS: usize, const COLS: usize>(pub [[T; COLS]; ROWS]);

pub type Matrix3x3<T> = StaticMatrix<T, 3, 3>;

impl<T, const ROWS: usize, const COLS: usize> StaticMatrix<T, ROWS, COLS> {
    pub fn num_rows(&self) -> usize {
        ROWS
    }

    pub fn num_cols(&self) -> usize {
        COLS
    }

    pub fn dim(&self) -> (usize, usize) {
        (self.num_rows(), self.num_cols())
    }

    pub fn get_row(&self, row: usize) -> Option<StaticVector<T, COLS>>
    where
        T: Copy,
    {
        Some(StaticVector(*self.0.get(row)?))
    }

    pub fn get_col(&self, col: usize) -> Option<StaticVector<T, ROWS>>
    where
        T: ConstZero + Copy,
    {
        if col >= COLS {
            return None;
        }

        let mut arr = [T::ZERO; ROWS];
        for (i, row) in self.0.iter().enumerate() {
            arr[i] = row[col];
        }

        Some(StaticVector(arr))
    }

    pub fn transpose(self) -> StaticMatrix<T, COLS, ROWS>
    where
        T: ConstZero + Copy + PartialEq,
    {
        let mut ret: StaticMatrix<T, COLS, ROWS> = StaticMatrix::zero();
        for (i, row) in ret.0.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                *item = self.0[j][i];
            }
        }

        ret
    }
}

impl<T, const SIZE: usize> StaticMatrix<T, SIZE, SIZE> {
    pub fn identity() -> Self
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
    T: Copy + PartialEq + Zero,
{
    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    fn set_zero(&mut self) {
        *self = Self::zero()
    }

    fn zero() -> Self {
        StaticMatrix([[T::zero(); COLS]; ROWS])
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

impl<T, const X: usize, const Y: usize, const Z: usize> core::ops::Mul<StaticMatrix<T, Y, Z>>
    for StaticMatrix<T, X, Y>
where
    T: ConstZero + Copy + PartialEq + core::ops::Mul<Output = T>,
{
    type Output = StaticMatrix<T, X, Z>;

    fn mul(self, rhs: StaticMatrix<T, Y, Z>) -> Self::Output {
        let mut ret = StaticMatrix::ZERO;

        for (i, row) in ret.0.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                *item = self.get_row(i).unwrap().dot(&rhs.get_col(j).unwrap());
            }
        }

        ret
    }
}

impl<T, const SIZE: usize> core::ops::MulAssign for StaticMatrix<T, SIZE, SIZE>
where
    T: ConstZero + Copy + PartialEq + core::ops::Mul<Output = T>{
    fn mul_assign(&mut self, rhs: Self) {
        let clone = self.clone();

        for (i, row) in self.0.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                *item = clone.get_row(i).unwrap().dot(&rhs.get_col(j).unwrap());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::id;

    use crate::{matrix::Matrix3x3, vector::StaticVector};

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

    #[test]
    fn matrix_identity_transpose() {
        let identity3x3: Matrix3x3<i32> = StaticMatrix::identity();
        assert_eq!(identity3x3.clone(), identity3x3.transpose());
    }

    #[test]
    fn matrix_square_transpose() {
        let mat = StaticMatrix([[1, 2], [3, 4]]);
        let mat_t = StaticMatrix([[1, 3], [2, 4]]);
        assert_eq!(mat.transpose(), mat_t);
    }

    #[test]
    fn matrix_rect_transpose() {
        let mat = StaticMatrix([[1, 2], [3, 4], [5, 6]]);
        let mat_t = StaticMatrix([[1, 3, 5], [2, 4, 6]]);
        assert_eq!(mat.transpose(), mat_t);
    }

    #[test]
    fn matrix_get_row() {
        let mat = StaticMatrix([[1, 2], [3, 4]]);
        assert_eq!(mat.get_row(0), Some(StaticVector([1, 2])));
    }

    #[test]
    fn matrix_get_col() {
        let mat = StaticMatrix([[1, 2], [3, 4]]);
        assert_eq!(mat.get_col(1), Some(StaticVector([2, 4])));
    }

    #[test]
    fn mat_mul_identity() {
        let identity3x3: Matrix3x3<i32> = StaticMatrix::identity();
        assert_eq!(identity3x3.clone() * identity3x3.clone(), identity3x3);
    }

    #[test]
    fn mat_mul_square() {
        let mat_a = StaticMatrix([[1, 2], [3, 4]]);
        let mat_b = StaticMatrix([[1, -1], [-1, 1]]);
        let mat_res = StaticMatrix([[-1, 1], [-1, 1]]);
        assert_eq!(mat_a.clone() * mat_b.clone(), mat_res.clone());
        assert_eq!(mat_b.transpose() * mat_a.transpose(), mat_res.transpose());
    }

    #[test]
    fn mat_mul_rect() {
        let mat_a = StaticMatrix([[1, 2]]);
        let mat_b = StaticMatrix([[1, -1], [-1, 1]]);
        let mat_res = StaticMatrix([[-1, 1]]);
        assert_eq!(mat_a * mat_b, mat_res);
    }
}
