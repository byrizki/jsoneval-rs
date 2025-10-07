use serde_json::Value;
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

/// SmallVec for path segments - avoids heap allocation for common cases (<= 4 segments)
pub type PathVec = SmallVec<[PathSegment; 4]>;

#[inline]
pub fn parse_path(path: &str) -> PathVec {
    let mut segments = PathVec::new();
    if path.is_empty() {
        return segments;
    }

    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Ok(idx) = part.parse::<usize>() {
            segments.push(PathSegment::Index(idx));
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }

    segments
}

#[inline]
pub fn traverse<'a>(value: &'a Value, segments: &[PathSegment]) -> Option<&'a Value> {
    let mut current = value;
    for segment in segments {
        match (segment, current) {
            (PathSegment::Key(key), Value::Object(map)) => {
                current = map.get(key)?;
            }
            (PathSegment::Index(index), Value::Array(arr)) => {
                current = arr.get(*index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

#[inline]
pub fn traverse_mut<'a>(value: &'a mut Value, segments: &[PathSegment]) -> Option<&'a mut Value> {
    let mut current = value;
    for segment in segments {
        match (segment, current) {
            (PathSegment::Key(key), Value::Object(map)) => {
                current = map.get_mut(key)?;
            }
            (PathSegment::Index(index), Value::Array(arr)) => {
                current = arr.get_mut(*index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}
