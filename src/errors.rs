//! Error types returned by fallible triangulation operations.

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Creates an [InternalError] for a violated internal invariant.
///
/// Triggers a `debug_assert!` failure, so that the bug is caught early in debug builds and
/// tests. Release builds report the error to the caller instead of panicking.
macro_rules! internal_error {
    ($message:literal) => {{
        debug_assert!(false, $message);
        $crate::InternalError::new($message)
    }};
}

/// An internal invariant of spade was violated.
///
/// This error always indicates a bug in spade. Please consider reporting it at
/// <https://github.com/Stoeoef/spade/issues>, ideally including the input that triggered it.
///
/// The operation that returned this error may have been interrupted midway. The triangulation
/// it was operating on may be inconsistent and should not be used anymore.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct InternalError {
    message: &'static str,
}

impl InternalError {
    pub(crate) const fn new(message: &'static str) -> Self {
        Self { message }
    }

    /// Returns a description of the violated invariant.
    pub fn message(&self) -> &'static str {
        self.message
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "internal error: {}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InternalError {}

/// The error type used for inserting elements into a triangulation.
///
/// Errors during insertion usually originate from an invalid vertex position. Vertices can
/// be checked for validity by using [validate_vertex](crate::validate_vertex).
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum InsertionError {
    /// A coordinate value was too small.
    ///
    /// The absolute value of any inserted vertex coordinate must either be zero or greater
    /// than or equal to [MIN_ALLOWED_VALUE](crate::MIN_ALLOWED_VALUE).
    TooSmall,

    /// A coordinate value was too large.
    ///
    /// The absolute value of any inserted vertex coordinate must be less than or equal to
    /// [MAX_ALLOWED_VALUE](crate::MAX_ALLOWED_VALUE).
    TooLarge,

    /// A coordinate value was NaN.
    NAN,

    /// The triangulation cannot hold any more vertices.
    ///
    /// Element handles are stored as `u32`. This limits the number of vertices to
    /// roughly `u32::MAX / 6`, as every vertex adds up to 6 directed edges.
    TooManyVertices,

    /// An internal invariant was violated. See [InternalError].
    Internal(InternalError),
}

impl From<InternalError> for InsertionError {
    fn from(error: InternalError) -> Self {
        InsertionError::Internal(error)
    }
}

impl From<QueryError> for InsertionError {
    fn from(error: QueryError) -> Self {
        match error {
            QueryError::NAN => InsertionError::NAN,
            QueryError::InvalidRadius => InsertionError::Internal(internal_error!(
                "Radius query during insertion. This is a bug."
            )),
            QueryError::Internal(error) => InsertionError::Internal(error),
        }
    }
}

impl Display for InsertionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            InsertionError::Internal(error) => Display::fmt(error, f),
            other => Debug::fmt(other, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InsertionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InsertionError::Internal(error) => Some(error),
            _ => None,
        }
    }
}

/// The error type returned when looking up positions or shapes within a triangulation.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum QueryError {
    /// A query coordinate was NaN.
    NAN,

    /// A query radius was negative or NaN.
    InvalidRadius,

    /// An internal invariant was violated. See [InternalError].
    Internal(InternalError),
}

impl From<InternalError> for QueryError {
    fn from(error: InternalError) -> Self {
        QueryError::Internal(error)
    }
}

impl Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            QueryError::NAN => f.write_str("a query coordinate was NaN"),
            QueryError::InvalidRadius => f.write_str("the query radius was negative or NaN"),
            QueryError::Internal(error) => Display::fmt(error, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryError::Internal(error) => Some(error),
            _ => None,
        }
    }
}

/// The error type returned when removing a vertex from a triangulation.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum RemovalError {
    /// The given vertex handle does not refer to a vertex of the triangulation.
    InvalidHandle,

    /// A query coordinate was NaN.
    NAN,

    /// An internal invariant was violated. See [InternalError].
    Internal(InternalError),
}

impl From<InternalError> for RemovalError {
    fn from(error: InternalError) -> Self {
        RemovalError::Internal(error)
    }
}

impl From<QueryError> for RemovalError {
    fn from(error: QueryError) -> Self {
        match error {
            QueryError::NAN => RemovalError::NAN,
            QueryError::InvalidRadius => RemovalError::Internal(internal_error!(
                "Radius query during removal. This is a bug."
            )),
            QueryError::Internal(error) => RemovalError::Internal(error),
        }
    }
}

impl Display for RemovalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RemovalError::InvalidHandle => f.write_str("the vertex handle is invalid"),
            RemovalError::NAN => f.write_str("a query coordinate was NaN"),
            RemovalError::Internal(error) => Display::fmt(error, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RemovalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RemovalError::Internal(error) => Some(error),
            _ => None,
        }
    }
}

/// The error type returned when adding a constraint edge fails.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum AddConstraintError {
    /// The new constraint edge would intersect an existing constraint edge.
    ///
    /// The triangulation is left unchanged. Use
    /// [ConstrainedDelaunayTriangulation::add_constraint_and_split](crate::ConstrainedDelaunayTriangulation::add_constraint_and_split)
    /// to split the intersected constraint edges instead.
    IntersectsConstraintEdge,

    /// Inserting a vertex of the constraint edge failed.
    Insertion(InsertionError),

    /// An internal invariant was violated. See [InternalError].
    Internal(InternalError),
}

impl From<InsertionError> for AddConstraintError {
    fn from(error: InsertionError) -> Self {
        match error {
            InsertionError::Internal(error) => AddConstraintError::Internal(error),
            error => AddConstraintError::Insertion(error),
        }
    }
}

impl From<InternalError> for AddConstraintError {
    fn from(error: InternalError) -> Self {
        AddConstraintError::Internal(error)
    }
}

impl From<QueryError> for AddConstraintError {
    fn from(error: QueryError) -> Self {
        InsertionError::from(error).into()
    }
}

impl Display for AddConstraintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AddConstraintError::IntersectsConstraintEdge => {
                f.write_str("the new constraint edge intersects an existing constraint edge")
            }
            AddConstraintError::Insertion(error) => write!(f, "vertex insertion failed: {error}"),
            AddConstraintError::Internal(error) => Display::fmt(error, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AddConstraintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AddConstraintError::IntersectsConstraintEdge => None,
            AddConstraintError::Insertion(error) => Some(error),
            AddConstraintError::Internal(error) => Some(error),
        }
    }
}

/// The error type returned when bulk loading a constrained Delaunay triangulation.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[non_exhaustive]
pub enum BulkLoadCdtError {
    /// A vertex could not be inserted.
    Insertion(InsertionError),

    /// A constraint edge refers to a vertex index that is out of range.
    InvalidEdgeIndex([usize; 2]),

    /// A constraint edge intersects another constraint edge.
    ///
    /// Contains the vertex indices of the edge that could not be added. Use
    /// [ConstrainedDelaunayTriangulation::try_bulk_load_cdt](crate::ConstrainedDelaunayTriangulation::try_bulk_load_cdt)
    /// to skip conflicting edges instead.
    ConflictingEdge([usize; 2]),

    /// An internal invariant was violated. See [InternalError].
    Internal(InternalError),
}

impl From<InsertionError> for BulkLoadCdtError {
    fn from(error: InsertionError) -> Self {
        match error {
            InsertionError::Internal(error) => BulkLoadCdtError::Internal(error),
            error => BulkLoadCdtError::Insertion(error),
        }
    }
}

impl From<InternalError> for BulkLoadCdtError {
    fn from(error: InternalError) -> Self {
        BulkLoadCdtError::Internal(error)
    }
}

impl From<AddConstraintError> for BulkLoadCdtError {
    fn from(error: AddConstraintError) -> Self {
        match error {
            AddConstraintError::Insertion(error) => error.into(),
            AddConstraintError::Internal(error) => error.into(),
            AddConstraintError::IntersectsConstraintEdge => {
                BulkLoadCdtError::Internal(internal_error!(
                    "Unexpected constraint conflict during bulk loading. This is a bug."
                ))
            }
        }
    }
}

impl Display for BulkLoadCdtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BulkLoadCdtError::Insertion(error) => write!(f, "vertex insertion failed: {error}"),
            BulkLoadCdtError::InvalidEdgeIndex([from, to]) => {
                write!(
                    f,
                    "constraint edge [{from}, {to}] refers to an invalid vertex index"
                )
            }
            BulkLoadCdtError::ConflictingEdge([from, to]) => {
                write!(
                    f,
                    "constraint edge [{from}, {to}] intersects another constraint edge"
                )
            }
            BulkLoadCdtError::Internal(error) => Display::fmt(error, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BulkLoadCdtError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BulkLoadCdtError::Insertion(error) => Some(error),
            BulkLoadCdtError::Internal(error) => Some(error),
            _ => None,
        }
    }
}

/// The error type returned when creating a handle from an index that cannot be represented.
///
/// Handles store their index as `u32`.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct HandleIndexError {
    index: usize,
}

impl HandleIndexError {
    pub(crate) const fn new(index: usize) -> Self {
        Self { index }
    }

    /// Returns the index that could not be converted into a handle.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Display for HandleIndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "handle index {} exceeds u32::MAX", self.index)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HandleIndexError {}
