use core::cmp::Ordering;

pub enum Solution<T> {
    One(T),
    Two([T; 2]),
}

impl<T> IntoIterator for Solution<T> {
    type Item = T;
    type IntoIter = SolutionIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Solution::One(a) => SolutionIter::One(a),
            Solution::Two(a) => SolutionIter::Two(a),
        }
    }
}

pub enum SolutionIter<T> {
    Exhausted,
    One(T),
    Two([T; 2]),
}

impl<T> From<Option<Solution<T>>> for SolutionIter<T> {
    fn from(value: Option<Solution<T>>) -> Self {
        match value {
            None => SolutionIter::Exhausted,
            Some(Solution::One(a)) => SolutionIter::One(a),
            Some(Solution::Two(a)) => SolutionIter::Two(a),
        }
    }
}

impl<T> Iterator for SolutionIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match core::mem::replace(self, Self::Exhausted) {
            Self::Exhausted => None,
            Self::One(x) => {
                *self = Self::Exhausted;
                Some(x)
            }
            Self::Two([a, b]) => {
                *self = Self::One(a);
                Some(b)
            }
        }
    }
}

// solve ax + b = 0
pub fn linear(a: f32, b: f32) -> Option<f32> {
    if a == 0.0 {
        None
    } else {
        Some(-b / a)
    }
}

/// Solve ax^2 + bx + c = 0
pub fn quadratic(a: f32, b: f32, c: f32) -> Option<Solution<f32>> {
    if a == 0.0 {
        return linear(b, c).map(Solution::One);
    }
    
    let discriminant = b * b - 4.0 * a * c;
    
    match discriminant.partial_cmp(&0.0)? {
        Ordering::Less => None,
        Ordering::Equal => Some(Solution::One(-b / (2.0 * a))),
        Ordering::Greater => {
            let discriminant_sqrt = bevy_math::ops::sqrt(b * b - 4.0 * a * c);
            let r_1 = (-b + discriminant_sqrt) / (2.0 * a);
            let r_2 = (-b - discriminant_sqrt) / (2.0 * a);
            Some(Solution::Two([r_1, r_2]))
        }
    }
}