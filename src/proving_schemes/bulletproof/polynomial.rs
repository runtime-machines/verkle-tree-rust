use curve25519_dalek::scalar::Scalar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial(pub(crate) Vec<Scalar>);

impl Polynomial {
    pub fn new(coefficients: Vec<Scalar>) -> Self {
        let mut poly = Polynomial(coefficients);
        poly.normalize();
        poly
    }

    pub fn from(coefficients: &[u128]) -> Self {
        Polynomial::new(
            coefficients
                .iter()
                .map(|n| Scalar::from(*n))
                .collect::<Vec<Scalar>>(),
        )
    }

    pub fn zero() -> Self {
        Polynomial(vec![Scalar::zero()])
    }

    pub fn one() -> Self {
        Polynomial(vec![Scalar::one()])
    }

    pub fn degree(&self) -> usize {
        self.0.len() - 1
    }

    // TODO: check correctness of numerator
    // Creates a polinomial that contains a set of `p` points, by using lagrange
    pub fn lagrange(p: &[(Scalar, Scalar)]) -> Self {
        let k = p.len();
        let mut l = Polynomial::zero();
        for j in 0..k {
            let mut l_j = Polynomial::one();
            for i in 0..k {
                if i != j {
                    let c = (p[j].0 - p[i].0).invert();
                    l_j = &l_j * &Polynomial::new(vec![-(c * p[i].0), c]);
                }
            }
            l += &(&l_j * &p[j].1);
        }
        l
    }

    // Remove ending zeroes
    pub fn normalize(&mut self) {
        if self.0.len() > 1 && self.0[self.0.len() - 1] == Scalar::zero() {
            let zero = Scalar::zero();
            let first_non_zero = self.0.iter().rev().position(|p| p != &zero);
            if let Some(first_non_zero) = first_non_zero {
                self.0.resize(self.0.len() - first_non_zero, Scalar::zero());
            } else {
                self.0.resize(1, Scalar::zero());
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        self.0.len() == 1 && self.0[0] == Scalar::zero()
    }

    pub fn set(&mut self, i: usize, p: Scalar) {
        if self.0.len() < i + 1 {
            self.0.resize(i + 1, Scalar::zero());
        }
        self.0[i] = p;
        self.normalize();
    }

    pub fn get(&mut self, i: usize) -> Option<&Scalar> {
        self.0.get(i)
    }
}

impl std::ops::AddAssign<&Polynomial> for Polynomial {
    fn add_assign(&mut self, rhs: &Polynomial) {
        for n in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n]);
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] += rhs.0[n];
            }
        }
        self.normalize();
    }
}

impl std::ops::AddAssign<&Scalar> for Polynomial {
    fn add_assign(&mut self, rhs: &Scalar) {
        self.0[0] += rhs;
    }
}

impl std::ops::SubAssign<&Polynomial> for Polynomial {
    fn sub_assign(&mut self, rhs: &Polynomial) {
        for n in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n]);
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] -= rhs.0[n];
            }
        }
        self.normalize();
    }
}

impl std::ops::Mul<&Polynomial> for &Polynomial {
    type Output = Polynomial;
    fn mul(self, rhs: &Polynomial) -> Self::Output {
        let mut mul: Vec<Scalar> = std::iter::repeat(Scalar::zero())
            .take(self.0.len() + rhs.0.len() - 1)
            .collect();
        for n in 0..self.0.len() {
            for m in 0..rhs.0.len() {
                mul[n + m] += self.0[n] * rhs.0[m];
            }
        }
        Polynomial(mul)
    }
}

impl std::ops::Mul<&Scalar> for &Polynomial {
    type Output = Polynomial;
    fn mul(self, rhs: &Scalar) -> Self::Output {
        if rhs == &Scalar::zero() {
            Polynomial::zero()
        } else {
            Polynomial(self.0.iter().map(|v| v * rhs).collect::<Vec<_>>())
        }
    }
}

impl std::ops::Div for Polynomial {
    type Output = (Polynomial, Polynomial);

    fn div(self, rhs: Polynomial) -> Self::Output {
        let (mut q, mut r) = (Polynomial::zero(), self);
        while !r.is_zero() && r.degree() >= rhs.degree() {
            let lead_r = r.0[r.0.len() - 1];
            let lead_d = rhs.0[rhs.0.len() - 1];
            let mut t = Polynomial::zero();
            t.set(r.0.len() - rhs.0.len(), lead_r * lead_d.invert());
            q += &t;
            r -= &(&rhs * &t);
        }
        (q, r)
    }
}
