use num_traits::{ConstZero, Float, Zero};

use crate::matrix::StaticMatrix;

#[derive(Clone, Debug, PartialEq)]
pub struct StaticVector<T, const SIZE: usize>([T; SIZE]);

pub type Vector2D<T> = StaticVector<T, 2>;
pub type Vector3D<T> = StaticVector<T, 3>;

impl<T, const SIZE: usize> StaticVector<T, SIZE> {
    /// Returns the norm squared of the vector.
    pub fn get_norm2(&self) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        self.dot(&self)
    }

    /// Returns the norm of the vector.
    pub fn get_norm(&self) -> T
    where
        T: Float,
    {
        self.get_norm2().sqrt()
    }

    /// Compute the dot product between two vectors.
    pub fn dot(&self, rhs: &Self) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        self.0
            .iter()
            .zip(rhs.0.iter())
            .fold(T::zero(), |acc, (&l, &r)| acc + l * r)
    }

    /// Normalize this vector such that it has a norm of 1.
    ///
    /// Returns Err when trying to normalize the zero vector.
    pub fn normalize(&mut self) -> Result<(), String>
    where
        T: Float + core::ops::MulAssign,
    {
        let norm = self.get_norm();
        if norm == T::zero() {
            return Err(String::from("Caught division by Zero during normalization"));
        }
        *self *= T::one() / norm;
        Ok(())
    }

    /// Returns a unit vector pointing in the same direction as this vector
    pub fn unit(&self) -> Result<Self, String>
    where
        T: Float + core::ops::MulAssign,
    {
        let mut ret = self.clone();
        ret.normalize()?;

        Ok(ret)
    }
}

impl<T> StaticVector<T, 3> {
    /// Compute the cross product of two vectors.
    pub fn cross(lhs: &Self, rhs: &Self) -> Self
    where
        T: Float + core::ops::Add<T, Output = T> + core::ops::Mul<T, Output = T>,
    {
        StaticVector([
            lhs[1] * rhs[2] - lhs[2] * rhs[1],
            lhs[2] * rhs[0] - lhs[0] * rhs[2],
            lhs[0] * rhs[1] - lhs[1] * rhs[0],
        ])
    }
}

impl<T, const SIZE: usize> From<[T; SIZE]> for StaticVector<T, SIZE> {
    fn from(value: [T; SIZE]) -> Self {
        Self(value)
    }
}

impl<T, const SIZE: usize> ConstZero for StaticVector<T, SIZE>
where
    T: ConstZero + Copy + PartialEq,
{
    const ZERO: Self = StaticVector([T::ZERO; SIZE]);
}

impl<T, const SIZE: usize> Zero for StaticVector<T, SIZE>
where
    T: Copy + PartialEq + Zero,
{
    fn zero() -> Self {
        Self([T::zero(); SIZE])
    }

    fn set_zero(&mut self) {
        *self = Self::zero()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

/// Negating a vector reverses its direction.
impl<T, const SIZE: usize> core::ops::Neg for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Neg<Output = T>,
{
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = -*item;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Index<usize> for StaticVector<T, SIZE> {
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl<T, const SIZE: usize> core::ops::IndexMut<usize> for StaticVector<T, SIZE> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl<T, const SIZE: usize> core::ops::AddAssign<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::AddAssign<T>,
{
    fn add_assign(&mut self, rhs: T) {
        for item in self.0.iter_mut() {
            *item += rhs;
        }
    }
}

impl<T, const SIZE: usize> core::ops::AddAssign<&Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::AddAssign<T>,
{
    fn add_assign(&mut self, rhs: &Self) {
        for (l, r) in self.0.iter_mut().zip(rhs.0.iter()) {
            *l += *r
        }
    }
}

impl<T, const SIZE: usize> core::ops::AddAssign<Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::AddAssign<T>,
{
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T, const SIZE: usize> core::ops::SubAssign<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: T) {
        for item in self.0.iter_mut() {
            *item -= rhs
        }
    }
}

impl<T, const SIZE: usize> core::ops::SubAssign<&Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: &Self) {
        for (l, r) in self.0.iter_mut().zip(rhs.0.iter()) {
            *l -= *r
        }
    }
}

impl<T, const SIZE: usize> core::ops::SubAssign<Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::SubAssign<T>,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs
    }
}

impl<T, const SIZE: usize> core::ops::MulAssign<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::MulAssign<T>,
{
    fn mul_assign(&mut self, rhs: T) {
        for item in self.0.iter_mut() {
            *item *= rhs
        }
    }
}

/// Matrix multiplication.
impl<T, const SIZE: usize> core::ops::MulAssign<&StaticMatrix<T, SIZE, SIZE>>
    for StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    fn mul_assign(&mut self, rhs: &StaticMatrix<T, SIZE, SIZE>) {
        let clone = self.clone();

        for col in 0..SIZE {
            self.0[col] = clone.dot(&rhs.get_col(col).unwrap());
        }
    }
}

impl<T, const SIZE: usize> core::ops::MulAssign<StaticMatrix<T, SIZE, SIZE>>
    for StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    fn mul_assign(&mut self, rhs: StaticMatrix<T, SIZE, SIZE>) {
        *self *= &rhs;
    }
}

impl<T, const SIZE: usize> core::ops::Add<T> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(self, rhs: T) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T, const SIZE: usize> core::ops::Add<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(mut self, rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item + rhs;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Add<&StaticVector<T, SIZE>> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(self, rhs: &StaticVector<T, SIZE>) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T, const SIZE: usize> core::ops::Add<StaticVector<T, SIZE>> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(self, rhs: StaticVector<T, SIZE>) -> Self::Output {
        self.clone() + &rhs
    }
}

impl<T, const SIZE: usize> core::ops::Add<&StaticVector<T, SIZE>> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(mut self, rhs: &StaticVector<T, SIZE>) -> Self::Output {
        for (l, r) in self.0.iter_mut().zip(rhs.0.iter()) {
            *l = *l + *r;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Add<StaticVector<T, SIZE>> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn add(self, rhs: StaticVector<T, SIZE>) -> Self::Output {
        self + &rhs
    }
}

impl<T, const SIZE: usize> core::ops::Sub<T> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(self, rhs: T) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T, const SIZE: usize> core::ops::Sub<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(mut self, rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item - rhs;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Sub<&StaticVector<T, SIZE>> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(self, rhs: &StaticVector<T, SIZE>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T, const SIZE: usize> core::ops::Sub<StaticVector<T, SIZE>> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(self, rhs: StaticVector<T, SIZE>) -> Self::Output {
        self.clone() - &rhs
    }
}

impl<T, const SIZE: usize> core::ops::Sub<&StaticVector<T, SIZE>> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(mut self, rhs: &StaticVector<T, SIZE>) -> Self::Output {
        for (l, r) in self.0.iter_mut().zip(rhs.0.iter()) {
            *l = *l - *r;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Sub<StaticVector<T, SIZE>> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn sub(self, rhs: StaticVector<T, SIZE>) -> Self::Output {
        self - &rhs
    }
}

impl<T, const SIZE: usize> core::ops::Mul<T> for &StaticVector<T, SIZE>
where
    T: Copy + core::ops::Mul<T, Output = T>,
{
    type Output = StaticVector<T, SIZE>;

    fn mul(self, rhs: T) -> Self::Output {
        self.clone() * rhs
    }
}

impl<T, const SIZE: usize> core::ops::Mul<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Mul<T, Output = T>,
{
    type Output = Self;

    fn mul(mut self, rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item * rhs;
        }
        self
    }
}

impl<T, const COLS: usize, const SIZE: usize> core::ops::Mul<&StaticMatrix<T, SIZE, COLS>>
    for &StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    type Output = StaticVector<T, COLS>;

    fn mul(self, rhs: &StaticMatrix<T, SIZE, COLS>) -> Self::Output {
        let mut ret = [T::zero(); COLS];

        for col in 0..COLS {
            ret[col] = self.dot(&rhs.get_col(col).unwrap());
        }

        StaticVector::from(ret)
    }
}

impl<T, const COLS: usize, const SIZE: usize> core::ops::Mul<StaticMatrix<T, SIZE, COLS>>
    for &StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    type Output = StaticVector<T, COLS>;

    fn mul(self, rhs: StaticMatrix<T, SIZE, COLS>) -> Self::Output {
        self * &rhs
    }
}

impl<T, const COLS: usize, const SIZE: usize> core::ops::Mul<&StaticMatrix<T, SIZE, COLS>>
    for StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    type Output = StaticVector<T, COLS>;

    fn mul(self, rhs: &StaticMatrix<T, SIZE, COLS>) -> Self::Output {
        &self * rhs
    }
}

impl<T, const COLS: usize, const SIZE: usize> core::ops::Mul<StaticMatrix<T, SIZE, COLS>>
    for StaticVector<T, SIZE>
where
    T: Zero + Copy + core::ops::Mul<T, Output = T>,
{
    type Output = StaticVector<T, COLS>;

    fn mul(self, rhs: StaticMatrix<T, SIZE, COLS>) -> Self::Output {
        &self * &rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::StaticMatrix;

    use super::StaticVector;
    use num_traits::{ConstZero, Float};

    fn within_epsilon<T: Float, const SIZE: usize>(
        vec_expected: &StaticVector<T, SIZE>,
        vec_result: &StaticVector<T, SIZE>,
        eps: T,
    ) -> bool {
        vec_expected
            .0
            .iter()
            .zip(vec_result.0.iter())
            .all(|(&expected, &result)| (expected - result).abs() < eps)
    }

    #[test]
    fn vector_add_scalar_assign() {
        let mut vec = StaticVector([2, 4, 6]);
        vec += 1;
        assert_eq!(StaticVector([3, 5, 7]), vec);
    }

    #[test]
    fn vector_sub_scalar_assign() {
        let mut vec = StaticVector([2, 4, 6]);
        vec -= 1;
        assert_eq!(StaticVector([1, 3, 5]), vec);
    }

    #[test]
    fn vector_mul_scalar_assign() {
        let mut vec = StaticVector([2, 4, 6]);
        vec *= -9;
        assert_eq!(StaticVector([-18, -36, -54]), vec);
    }

    #[test]
    fn vector_mul_matrix_assign() {
        let mut vec = StaticVector([2, 4]);
        vec *= StaticMatrix::from([[1, -1], [-1, 3]]);
        assert_eq!(StaticVector([-2, 10]), vec);
    }

    #[test]
    fn vector_add_vector_assign() {
        let mut vec1 = StaticVector([2, 4, 6]);
        let vec2 = StaticVector([2, 7, 3]);
        vec1 += vec2;
        assert_eq!(StaticVector([4, 11, 9]), vec1);
    }

    #[test]
    fn vector_sub_vector_assign() {
        let mut vec1 = StaticVector([2, 4, 6]);
        let vec2 = StaticVector([2, 7, 3]);
        vec1 -= vec2;
        assert_eq!(StaticVector([0, -3, 3]), vec1);
    }

    #[test]
    fn vector_neg() {
        let mut vec = StaticVector([2, 4, 6]);
        vec = -vec;
        assert_eq!(StaticVector([-2, -4, -6]), vec);
    }

    #[test]
    fn vector_add_scalar() {
        let mut vec = StaticVector([2, 4, 6]);
        vec = vec + 1;
        assert_eq!(StaticVector([3, 5, 7]), vec);
    }

    #[test]
    fn vector_sub_scalar() {
        let mut vec = StaticVector([2, 4, 6]);
        vec = vec - 1;
        assert_eq!(StaticVector([1, 3, 5]), vec);
    }

    #[test]
    fn vector_mul_scalar() {
        let mut vec = StaticVector([2, 4, 6]);
        vec = vec * -9;
        assert_eq!(StaticVector([-18, -36, -54]), vec);
    }

    #[test]
    fn vector_mul_matrix() {
        let mut vec = StaticVector([2, 4]);
        vec = vec * StaticMatrix::from([[1, -1], [-1, 3]]);
        assert_eq!(StaticVector([-2, 10]), vec);
    }

    #[test]
    fn vector_add_vector() {
        let vec1 = StaticVector([2, 4, 6]);
        let vec2 = StaticVector([2, 7, 3]);
        let vec3 = vec1 + vec2;
        assert_eq!(StaticVector([4, 11, 9]), vec3);
    }

    #[test]
    fn vector_sub_vector() {
        let vec1 = StaticVector([2, 4, 6]);
        let vec2 = StaticVector([2, 7, 3]);
        let vec3 = vec1 - vec2;
        assert_eq!(StaticVector([0, -3, 3]), vec3);
    }

    #[test]
    fn vector_commutative() {
        let vec1 = StaticVector([0, 3]);
        let vec2 = StaticVector([-1, 1]);
        assert_eq!(vec1.clone() + vec2.clone(), vec2.clone() + vec1.clone())
    }

    #[test]
    fn vector_associative() {
        let vec1 = StaticVector([0, 3]);
        let vec2 = StaticVector([-1, 1]);
        let vec3 = StaticVector([-5, -3]);
        assert_eq!(
            vec1.clone() + (vec2.clone() + vec3.clone()),
            (vec1.clone() + vec2.clone()) + vec3.clone()
        )
    }

    #[test]
    fn vector_zero() {
        let vec = StaticVector([2, 2, 1]);
        assert_eq!(vec.clone() + StaticVector::ZERO, vec.clone());
    }

    #[test]
    fn vector_inverse() {
        let vec = StaticVector([2, 2, 1]);
        let vec_inv = -vec.clone();
        assert_eq!(vec.clone() + vec_inv.clone(), StaticVector::ZERO);
    }

    #[test]
    fn vector_unit_scale() {
        let vec = StaticVector([2, 2, 1]);
        assert_eq!(vec.clone() * 1, vec.clone());
    }

    #[test]
    fn vector_scalar_associativity() {
        let vec = StaticVector([2, 2, 1]);
        let a = 3;
        let b = 5;
        assert_eq!((vec.clone() * a) * b, vec.clone() * (a * b));
    }

    #[test]
    fn vector_scalar_scalar_distribution() {
        let vec1 = StaticVector([2, 2, 1]);
        let vec2 = StaticVector([-1, 0, 1]);
        let a = 3;
        assert_eq!(
            (vec1.clone() + vec2.clone()) * a,
            (vec1.clone() * a) + (vec2.clone() * a)
        );
    }

    #[test]
    fn vector_scalar_vector_distribution() {
        let vec = StaticVector([2, 2, 1]);
        let a = 3;
        let b = 5;
        assert_eq!(vec.clone() * (a + b), vec.clone() * a + vec.clone() * b);
    }

    #[test]
    fn vector_norm() {
        let vec = StaticVector([-3.0, 4.0]);
        assert_eq!(5.0, vec.get_norm());
    }

    #[test]
    fn vector_norm2() {
        let vec = StaticVector([3.0, -4.0]);
        assert_eq!(25.0, vec.get_norm2());
    }

    #[test]
    fn vector_normalize() {
        let mut vec = StaticVector([3.0, -4.0]);
        vec.normalize().unwrap();
        assert!(within_epsilon(
            &StaticVector([0.6, -0.8]),
            &vec,
            f64::EPSILON
        ));
    }

    #[test]
    #[should_panic]
    fn vector_normalize_zero() {
        let mut vec: StaticVector<f64, 3> = StaticVector::ZERO;
        vec.normalize().unwrap()
    }

    #[test]
    fn vector_unit_vec() {
        let vec = StaticVector([3.0, -4.0]);
        let unit_vec = vec.unit().unwrap();
        assert!(within_epsilon(
            &StaticVector([0.6, -0.8]),
            &unit_vec,
            f64::EPSILON
        ));
    }

    #[test]
    fn vector_dot() {
        let vec1 = StaticVector([-1.0, -2.0, 3.0]);
        let vec2 = StaticVector([4.0, 0.0, -8.0]);
        assert_eq!(-28.0, vec1.dot(&vec2));
    }

    #[test]
    fn vector_cross() {
        let vec1 = StaticVector([-1.0, -2.0, 3.0]);
        let vec2 = StaticVector([4.0, 0.0, -8.0]);
        assert_eq!(
            StaticVector([16.0, 4.0, 8.0]),
            StaticVector::cross(&vec1, &vec2)
        );
    }
}
