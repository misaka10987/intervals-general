use crate::bound_pair::BoundPair;

/// Interval enum capable of general interval representation
///
/// Where applicable, using lower bound `a` and upper bound `b`.  An Interval taxonomy was pulled from [proofwiki](https://proofwiki.org/wiki/Definition:Real_Interval_Types).
///
/// * Closed -> `[a, b]`
/// * Open -> `(a,b)`
/// * LeftHalfOpen -> `(a, b]`
/// * RightHalfOpen -> `[a, b)`
/// * UnboundedClosedRight -> `(-inf, a]`
/// * UnboundedOpenRight -> `(-inf, a)`
/// * UnboundedClosedLeft -> `[a, inf)`
/// * UnboundedOpenLeft -> `(a, inf)`
/// * Singeleton -> `[a]`
/// * Unbounded -> `(-inf, inf)`
/// * Empty
///
/// # Examples
///
/// ```
/// use intervals_general::bound_pair::BoundPair;
/// use intervals_general::interval::Interval;
/// # fn main() -> std::result::Result<(), String> {
/// let bounds = BoundPair::new(1.0, 2.0).ok_or("invalid BoundPair")?;
/// let right_half_open = Interval::RightHalfOpen { bound_pair: bounds }; // [1.0, 2.0)
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Interval<T> {
    Closed { bound_pair: BoundPair<T> },
    Open { bound_pair: BoundPair<T> },
    LeftHalfOpen { bound_pair: BoundPair<T> },
    RightHalfOpen { bound_pair: BoundPair<T> },
    UnboundedClosedRight { right: T },
    UnboundedOpenRight { right: T },
    UnboundedClosedLeft { left: T },
    UnboundedOpenLeft { left: T },
    Singleton { at: T },
    Unbounded,
    Empty,
}

// Internally used to simplify matching functions on Intervals
// TODO(smoeller) do I need LeftBound and RightBound enums? Or just one.
// TODO(smoeller) drop Bounded from variant names
enum LeftBound<T> {
    None,
    Unbounded,
    OpenBounded(T),
    ClosedBounded(T),
}

// Internally used to simplify matching functions on Intervals
enum RightBound<T> {
    None,
    Unbounded,
    OpenBounded(T),
    ClosedBounded(T),
}

impl<T> Interval<T>
where
    T: Copy,
    T: std::cmp::PartialOrd,
    T: std::ops::Sub,
{
    /// Verify whether self contains the specified interval
    ///
    /// Interval I1.contains(I2) if and only if:
    ///
    /// * The left bound of I1 is bounded and less than or equal to the left
    ///   bound of I2 OR
    /// * the left bound of I1 is unbounded and the left bound of I2 is
    ///   unbounded
    ///
    /// AND
    ///
    /// * The right bound of I1 is bounded and greater than or equal to the
    ///   right bound of I2 OR
    /// * The right bound of I1 isunbounded and the left bound of I2 is
    ///   unbounded
    ///
    /// Additionally:
    ///
    /// * The Empty interval does not contain the Empty interval
    ///
    /// # Examples
    ///
    /// ```
    /// use intervals_general::bound_pair::BoundPair;
    /// use intervals_general::interval::Interval;
    /// # fn main() -> std::result::Result<(), String> {
    /// let right_half_open = Interval::RightHalfOpen {
    ///     bound_pair: BoundPair::new(1.0, 5.0).ok_or("invalid BoundPair")?,
    /// };
    /// let contained_interval = Interval::Open {
    ///     bound_pair: BoundPair::new(1.0, 2.0).ok_or("invalid BoundPair")?,
    /// };
    /// let non_contained_interval = Interval::Closed {
    ///     bound_pair: BoundPair::new(4.0, 5.0).ok_or("invalid BoundPair")?,
    /// };
    /// assert_eq!(right_half_open.contains(&contained_interval), true);
    /// assert_eq!(right_half_open.contains(&non_contained_interval), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains(&self, other: &Interval<T>) -> bool {
        let self_right_bound = self.to_right_bound();
        let other_right_bound = other.to_right_bound();
        let self_left_bound = self.to_left_bound();
        let other_left_bound = other.to_left_bound();

        let left_contained = match (self_left_bound, other_left_bound) {
            // The Empty interval does not contain the Empty interval
            (LeftBound::None, _) => false,
            (_, LeftBound::None) => false,
            // If self left interval is unbounded, it will contain any other left bound
            (LeftBound::Unbounded, _) => true,
            // Given self left interval is not unbounded and right is unbounded, self cannot contain
            // other
            (_, LeftBound::Unbounded) => false,
            (LeftBound::ClosedBounded(ref self_val), LeftBound::ClosedBounded(ref other_val))
            | (LeftBound::ClosedBounded(ref self_val), LeftBound::OpenBounded(ref other_val))
            | (LeftBound::OpenBounded(ref self_val), LeftBound::OpenBounded(ref other_val)) => {
                if self_val <= other_val {
                    true
                } else {
                    false
                }
            }
            (LeftBound::OpenBounded(ref self_val), LeftBound::ClosedBounded(ref other_val)) => {
                if self_val < other_val {
                    true
                } else {
                    false
                }
            }
        };

        let right_contained = match (self_right_bound, other_right_bound) {
            // The Empty interval does not contain the Empty interval
            (RightBound::None, _) => false,
            (_, RightBound::None) => false,
            // If self left interval is unbounded, it will contain any other left bound
            (RightBound::Unbounded, _) => true,
            // Given self left interval is not unbounded and right is unbounded, self cannot contain
            // other
            (_, RightBound::Unbounded) => false,
            (RightBound::ClosedBounded(ref self_val), RightBound::ClosedBounded(ref other_val))
            | (RightBound::ClosedBounded(ref self_val), RightBound::OpenBounded(ref other_val))
            | (RightBound::OpenBounded(ref self_val), RightBound::OpenBounded(ref other_val)) => {
                if self_val >= other_val {
                    true
                } else {
                    false
                }
            }
            (RightBound::OpenBounded(ref self_val), RightBound::ClosedBounded(ref other_val)) => {
                if self_val > other_val {
                    true
                } else {
                    false
                }
            }
        };

        left_contained && right_contained
    }

    fn to_left_bound(&self) -> LeftBound<T> {
        match self {
            Interval::Empty => LeftBound::None,
            Interval::Singleton { ref at } => LeftBound::ClosedBounded(*at),
            // The cases where left bound of self is open -inf
            Interval::Unbounded
            | Interval::UnboundedClosedRight { .. }
            | Interval::UnboundedOpenRight { .. } => LeftBound::Unbounded,
            // The cases where left bound of self is Closed and Bounded
            Interval::Closed {
                bound_pair: BoundPair { ref left, .. },
            }
            | Interval::RightHalfOpen {
                bound_pair: BoundPair { ref left, .. },
            }
            | Interval::UnboundedClosedLeft { ref left, .. } => LeftBound::ClosedBounded(*left),
            // The cases where left bound of self is Open and Bounded
            Interval::Open {
                bound_pair: BoundPair { ref left, .. },
            }
            | Interval::LeftHalfOpen {
                bound_pair: BoundPair { ref left, .. },
            }
            | Interval::UnboundedOpenLeft { ref left, .. } => LeftBound::OpenBounded(*left),
        }
    }

    fn to_right_bound(&self) -> RightBound<T> {
        match self {
            Interval::Empty => RightBound::None,
            Interval::Singleton { ref at } => RightBound::ClosedBounded(*at),
            // The cases where right bound of self is open +inf
            Interval::Unbounded
            | Interval::UnboundedClosedLeft { .. }
            | Interval::UnboundedOpenLeft { .. } => RightBound::Unbounded,
            // The cases where right bound of self is Closed and Bounded
            Interval::Closed {
                bound_pair: BoundPair { ref right, .. },
            }
            | Interval::LeftHalfOpen {
                bound_pair: BoundPair { ref right, .. },
            }
            | Interval::UnboundedClosedRight { ref right, .. } => {
                RightBound::ClosedBounded(*right)
            }
            // The cases where right bound of self is Open and Bounded
            Interval::Open {
                bound_pair: BoundPair { ref right, .. },
            }
            | Interval::RightHalfOpen {
                bound_pair: BoundPair { ref right, .. },
            }
            | Interval::UnboundedOpenRight { ref right, .. } => RightBound::OpenBounded(*right),
        }
    }
}