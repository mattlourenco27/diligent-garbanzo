use core::slice::Iter;
use num_traits::{ConstZero, Float, Zero};

pub trait Vector<T>:
    Clone
    + PartialEq
    + Zero
    + core::ops::Neg<Output = Self>
    + core::ops::Add<T, Output = Self>
    + core::ops::AddAssign<T>
    + core::ops::Add<Self, Output = Self>
    + core::ops::AddAssign<Self>
    + core::ops::Sub<T, Output = Self>
    + core::ops::SubAssign<T>
    + core::ops::Sub<Self, Output = Self>
    + core::ops::SubAssign<Self>
    + core::ops::Mul<T, Output = Self>
    + core::ops::MulAssign<T>
{
    fn iter(&self) -> Iter<T>;

    fn get_norm2(&self) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        ops::dot(self, self)
    }

    fn get_norm(&self) -> T
    where
        T: Float,
    {
        self.get_norm2().sqrt()
    }
}

pub mod ops {
    use num_traits::{Float, Zero};

    use super::Vector;

    pub fn dot<T>(_lhs: &impl Vector<T>, _rhs: &impl Vector<T>) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        _lhs.iter()
            .zip(_rhs.iter())
            .fold(T::zero(), |acc, (&l, &r)| acc + l * r)
    }

    pub fn normalize<T: Float>(vec: &mut impl Vector<T>) -> Result<(), String> {
        let norm = vec.get_norm();
        if norm == T::zero() {
            return Err(String::from("Caught division by Zero during normalization"));
        }
        *vec *= T::one() / norm;
        Ok(())
    }

    pub fn unit<T, U>(mut vec: T) -> Result<T, String>
    where
        T: Vector<U>,
        U: Float,
    {
        normalize(&mut vec)?;
        Ok(vec)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticVector<T, const SIZE: usize>(pub [T; SIZE]);

pub type Vector2D<T> = StaticVector<T, 2>;
pub type Vector3D<T> = StaticVector<T, 3>;

impl<T, const SIZE: usize> StaticVector<T, SIZE> {
    pub fn len(&self) -> usize {
        SIZE
    }
}

impl<T> StaticVector<T, 3> {
    pub fn cross(_lhs: &Self, _rhs: &Self) -> Self
    where
        T: Float + core::ops::Add<T, Output = T> + core::ops::Mul<T, Output = T>,
    {
        StaticVector([
            _lhs[1] * _rhs[2] - _lhs[2] * _rhs[1],
            _lhs[2] * _rhs[0] - _lhs[0] * _rhs[2],
            _lhs[0] * _rhs[1] - _lhs[1] * _rhs[0],
        ])
    }
}

impl<T, const SIZE: usize> Vector<T> for StaticVector<T, SIZE>
where
    T: ConstZero
        + Copy
        + PartialEq
        + core::ops::Neg<Output = T>
        + core::ops::Add<T, Output = T>
        + core::ops::AddAssign<T>
        + core::ops::Sub<T, Output = T>
        + core::ops::SubAssign<T>
        + core::ops::Mul<T, Output = T>
        + core::ops::MulAssign<T>,
{
    fn iter(&self) -> Iter<T> {
        self.0.iter()
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
    T: ConstZero + Copy + PartialEq,
{
    fn zero() -> Self {
        Self::ZERO
    }

    fn set_zero(&mut self) {
        *self = Self::ZERO
    }

    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

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
    fn add_assign(&mut self, _rhs: T) {
        for item in self.0.iter_mut() {
            *item += _rhs;
        }
    }
}

impl<T, const SIZE: usize> core::ops::AddAssign<Self> for StaticVector<T, SIZE>
where
    T: core::ops::AddAssign<T>,
{
    fn add_assign(&mut self, _rhs: Self) {
        for (l, r) in self.0.iter_mut().zip(_rhs.0.into_iter()) {
            *l += r
        }
    }
}

impl<T, const SIZE: usize> core::ops::SubAssign<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::SubAssign<T>,
{
    fn sub_assign(&mut self, _rhs: T) {
        for item in self.0.iter_mut() {
            *item -= _rhs
        }
    }
}

impl<T, const SIZE: usize> core::ops::SubAssign<Self> for StaticVector<T, SIZE>
where
    T: core::ops::SubAssign<T>,
{
    fn sub_assign(&mut self, _rhs: Self) {
        for (l, r) in self.0.iter_mut().zip(_rhs.0.into_iter()) {
            *l -= r
        }
    }
}

impl<T, const SIZE: usize> core::ops::MulAssign<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::MulAssign<T>,
{
    fn mul_assign(&mut self, _rhs: T) {
        for item in self.0.iter_mut() {
            *item *= _rhs
        }
    }
}

impl<T, const SIZE: usize> core::ops::Add<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = Self;

    fn add(mut self, _rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item + _rhs;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Add<Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Add<T, Output = T>,
{
    type Output = Self;

    fn add(mut self, _rhs: Self) -> Self::Output {
        for (l, r) in self.0.iter_mut().zip(_rhs.0.into_iter()) {
            *l = *l + r;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Sub<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(mut self, _rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item - _rhs;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Sub<Self> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(mut self, _rhs: Self) -> Self::Output {
        for (l, r) in self.0.iter_mut().zip(_rhs.0.into_iter()) {
            *l = *l - r;
        }
        self
    }
}

impl<T, const SIZE: usize> core::ops::Mul<T> for StaticVector<T, SIZE>
where
    T: Copy + core::ops::Mul<T, Output = T>,
{
    type Output = Self;

    fn mul(mut self, _rhs: T) -> Self::Output {
        for item in self.0.iter_mut() {
            *item = *item * _rhs;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{ops, StaticVector, Vector};
    use num_traits::{ConstZero, Float};

    fn within_epsilon<T: Float>(
        vec_expected: &impl Vector<T>,
        vec_result: &impl Vector<T>,
        eps: T,
    ) -> bool {
        vec_expected
            .iter()
            .zip(vec_result.iter())
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
        ops::normalize(&mut vec).unwrap();
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
        ops::normalize(&mut vec).unwrap()
    }

    #[test]
    fn vector_unit_vec() {
        let vec = StaticVector([3.0, -4.0]);
        let unit_vec = ops::unit(vec).unwrap();
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
        assert_eq!(-28.0, ops::dot(&vec1, &vec2));
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

    #[test]
    fn vector_len() {
        let vec = StaticVector([1, 2, 3, 4]);
        assert_eq!(4, vec.len());
    }
}
