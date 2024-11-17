use num_traits::{ConstOne, ConstZero, One, Zero};

use crate::vector::StaticVector;

#[derive(Clone, Debug, PartialEq)]
pub struct StaticMatrix<T, const ROWS: usize, const COLS: usize>([[T; COLS]; ROWS]);

pub type Matrix3x3<T> = StaticMatrix<T, 3, 3>;

impl<T, const ROWS: usize, const COLS: usize> StaticMatrix<T, ROWS, COLS> {
    /// Returns a copy of the specified row.
    ///
    /// Returns None if the index is greater than or equal to the ROWS hyperparameter.
    pub fn get_row(&self, row: usize) -> Option<StaticVector<T, COLS>>
    where
        T: Copy,
    {
        Some(StaticVector::from(*self.0.get(row)?))
    }

    /// Returns a copy of the specified column.
    ///
    /// Returns None if the index is greater than or equal to the COLS hyperparameter.
    pub fn get_col(&self, col: usize) -> Option<StaticVector<T, ROWS>>
    where
        T: Copy,
    {
        if col >= COLS {
            return None;
        }

        let tmp_vec: Vec<T> = self.0.iter().map(|row| row[col]).collect();
        let arr: [T; ROWS] = tmp_vec
            .try_into()
            .unwrap_or_else(|_| panic!("Expected number of elements equal to ROWS"));

        Some(arr.into())
    }

    /// Returns the transpose of this Matrix.
    ///
    /// Consumes self.
    pub fn transpose(self) -> StaticMatrix<T, COLS, ROWS>
    where
        T: Copy,
    {
        let tmp_vec: Vec<[T; ROWS]> = (0..COLS)
            .map(|col| {
                let tmp_vec: Vec<T> = self.0.iter().map(|row| row[col]).collect();
                tmp_vec
                    .try_into()
                    .unwrap_or_else(|_| panic!("Expected number of elements equal to ROWS"))
            })
            .collect();
        let arr: [[T; ROWS]; COLS] = tmp_vec
            .try_into()
            .unwrap_or_else(|_| panic!("Expected number of elements equal to COLS"));
        arr.into()
    }
}

impl<T, const SIZE: usize> StaticMatrix<T, SIZE, SIZE> {
    /// Constructs the identity for a matrix of size SIZE.
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

    /// Returns the transpose of this matrix.
    ///
    /// Much faster than the StaticMatrix::transpose() function but
    /// may only be used on square matrices.
    /// Consumes self.
    pub fn transpose_symmetric(mut self) -> Self
    where
        T: Copy,
    {
        if SIZE <= 1 {
            return self;
        }

        for i in 0..SIZE - 1 {
            for j in i + 1..SIZE {
                let tmp = self.0[i][j];
                self.0[i][j] = self.0[j][i];
                self.0[j][i] = tmp;
            }
        }

        self
    }
}

impl<T> Matrix3x3<T>
where
    T: ConstZero + ConstOne,
{
    pub const IDENTITY3X3: Self = Self([
        [T::ONE, T::ZERO, T::ZERO],
        [T::ZERO, T::ONE, T::ZERO],
        [T::ZERO, T::ZERO, T::ONE],
    ]);
}

impl<T, const ROWS: usize, const COLS: usize> From<[[T; COLS]; ROWS]>
    for StaticMatrix<T, ROWS, COLS>
{
    fn from(value: [[T; COLS]; ROWS]) -> Self {
        Self(value)
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

impl<T, const ROWS: usize, const COLS: usize> core::ops::Index<usize>
    for StaticMatrix<T, ROWS, COLS>
{
    type Output = [T; COLS];
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl<T, const ROWS: usize, const COLS: usize> core::ops::IndexMut<usize>
    for StaticMatrix<T, ROWS, COLS>
{
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

/// Element-wise addition of two matrices of the same size and dimension.
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

/// Matrix multiplication.
impl<T, const ROWS: usize, const SIZE: usize> core::ops::MulAssign<&StaticMatrix<T, SIZE, SIZE>>
    for StaticMatrix<T, ROWS, SIZE>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    fn mul_assign(&mut self, rhs: &StaticMatrix<T, SIZE, SIZE>) {
        let clone = self.clone();

        for (i, row) in self.0.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                *item = clone.get_row(i).unwrap().dot(&rhs.get_col(j).unwrap());
            }
        }
    }
}

impl<T, const ROWS: usize, const SIZE: usize> core::ops::MulAssign<StaticMatrix<T, SIZE, SIZE>>
    for StaticMatrix<T, ROWS, SIZE>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    fn mul_assign(&mut self, rhs: StaticMatrix<T, SIZE, SIZE>) {
        *self *= &rhs;
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> core::ops::Mul<&StaticMatrix<T, Y, Z>>
    for &StaticMatrix<T, X, Y>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticMatrix<T, X, Z>;

    fn mul(self, rhs: &StaticMatrix<T, Y, Z>) -> Self::Output {
        let mut ret = [[T::zero(); Z]; X];

        for (i, row) in ret.iter_mut().enumerate() {
            for (j, item) in row.iter_mut().enumerate() {
                *item = self.get_row(i).unwrap().dot(&rhs.get_col(j).unwrap());
            }
        }

        StaticMatrix::from(ret)
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> core::ops::Mul<StaticMatrix<T, Y, Z>>
    for &StaticMatrix<T, X, Y>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticMatrix<T, X, Z>;

    fn mul(self, rhs: StaticMatrix<T, Y, Z>) -> Self::Output {
        self * &rhs
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> core::ops::Mul<&StaticMatrix<T, Y, Z>>
    for StaticMatrix<T, X, Y>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticMatrix<T, X, Z>;

    fn mul(self, rhs: &StaticMatrix<T, Y, Z>) -> Self::Output {
        &self * rhs
    }
}

impl<T, const X: usize, const Y: usize, const Z: usize> core::ops::Mul<StaticMatrix<T, Y, Z>>
    for StaticMatrix<T, X, Y>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticMatrix<T, X, Z>;

    fn mul(self, rhs: StaticMatrix<T, Y, Z>) -> Self::Output {
        &self * &rhs
    }
}

/// Multiply by a vector.
/// 
/// Treat multiplication with a vector as if the vector was a column vector.
impl<T, const ROWS: usize, const COLS: usize> core::ops::Mul<&StaticVector<T, COLS>>
    for &StaticMatrix<T, ROWS, COLS>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticVector<T, ROWS>;

    fn mul(self, rhs: &StaticVector<T, COLS>) -> Self::Output {
        let mut ret = [T::zero(); ROWS];

        for (i, item) in ret.iter_mut().enumerate() {
            *item = self.get_row(i).unwrap().dot(&rhs);
        }

        StaticVector::from(ret)
    }
}

impl<T, const ROWS: usize, const COLS: usize> core::ops::Mul<StaticVector<T, COLS>>
    for &StaticMatrix<T, ROWS, COLS>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticVector<T, ROWS>;

    fn mul(self, rhs: StaticVector<T, COLS>) -> Self::Output {
        self * &rhs
    }
}

impl<T, const ROWS: usize, const COLS: usize> core::ops::Mul<&StaticVector<T, COLS>>
    for StaticMatrix<T, ROWS, COLS>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticVector<T, ROWS>;

    fn mul(self, rhs: &StaticVector<T, COLS>) -> Self::Output {
        &self * rhs
    }
}

impl<T, const ROWS: usize, const COLS: usize> core::ops::Mul<StaticVector<T, COLS>>
    for StaticMatrix<T, ROWS, COLS>
where
    T: Zero + Copy + core::ops::Mul<Output = T>,
{
    type Output = StaticVector<T, ROWS>;

    fn mul(self, rhs: StaticVector<T, COLS>) -> Self::Output {
        &self * &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::vector::StaticVector;

    use super::{StaticMatrix, Matrix3x3};

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
        let mat = StaticMatrix([[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
        let mat_t = StaticMatrix([[1, 4, 7], [2, 5, 8], [3, 6, 9]]);
        assert_eq!(mat.transpose_symmetric(), mat_t);
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
        assert_eq!(mat.get_row(0), Some([1, 2].into()));
    }

    #[test]
    fn matrix_get_col() {
        let mat = StaticMatrix([[1, 2], [3, 4]]);
        assert_eq!(mat.get_col(1), Some([2, 4].into()));
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
        assert_eq!(&mat_a * &mat_b, mat_res.clone());
        assert_eq!(mat_b.transpose() * mat_a.transpose(), mat_res.transpose());
    }

    #[test]
    fn mat_mul_rect() {
        let mat_a = StaticMatrix([[1, 2]]);
        let mat_b = StaticMatrix([[1, -1], [-1, 1]]);
        let mat_res = StaticMatrix([[-1, 1]]);
        assert_eq!(mat_a * mat_b, mat_res);
    }

    #[test]
    fn mat_vec_mul() {
        let mat = StaticMatrix([[1, 2, 3], [-1, -2, -3], [1, 0, -1]]);
        let vec = StaticVector::from([1, 2, -1]);
        let vec_res = StaticVector::from([2, -2, 2]);
        assert_eq!(mat * vec, vec_res);
    }
}
