use num_traits::{ConstZero, Float, Zero};

#[derive(Clone, Debug, PartialEq)]
pub struct StaticVector<T, const SIZE: usize>(pub [T; SIZE]);

pub type Vector2D<T> = StaticVector<T, 2>;
pub type Vector3D<T> = StaticVector<T, 3>;

impl<T, const SIZE: usize> StaticVector<T, SIZE> {
    pub fn len(&self) -> usize {
        SIZE
    }

    pub fn get_norm2(&self) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        self.dot(&self)
    }

    pub fn get_norm(&self) -> T
    where
        T: Float,
    {
        self.get_norm2().sqrt()
    }

    pub fn dot(&self, rhs: &Self) -> T
    where
        T: Zero + Copy + core::ops::Mul<T, Output = T>,
    {
        self.0
            .iter()
            .zip(rhs.0.iter())
            .fold(T::zero(), |acc, (&l, &r)| acc + l * r)
    }

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

    pub fn unit(mut self) -> Result<Self, String>
    where
        T: Float + core::ops::MulAssign,
    {
        self.normalize()?;
        Ok(self)
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
    use super::StaticVector;
    use num_traits::{ConstZero, Float};

    fn within_epsilon<T: Float, const SIZE: usize>(
        vec_expected: &StaticVector<T, SIZE>,
        vec_result: &StaticVector<T, SIZE>,
        eps: T,
    ) -> bool {
        vec_expected
            .0.iter()
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

    #[test]
    fn vector_len() {
        let vec = StaticVector([1, 2, 3, 4]);
        assert_eq!(4, vec.len());
    }
}
