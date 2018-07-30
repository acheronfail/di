use pretty_bytes;
use std::cmp::{max, Ordering};
use std::collections::BinaryHeap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Info(pub u64, pub PathBuf);

impl Display for Info {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let size = pretty_bytes::converter::convert(self.0 as f64);
        write!(f, "{size} {path}", size = size, path = self.1.display())
    }
}

// Implement `Eq`, `PartialEq`, `PartialOrd` and `Ord` for `Info` so we can
// turn the standard BinaryHeap into a min-heap.
impl Eq for Info {}

impl PartialEq for Info {
    fn eq(&self, other: &Info) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Info {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for Info {
    fn cmp(&self, other: &Info) -> Ordering {
        match self.partial_cmp(other).unwrap() {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

/// This `LimitedHeap` is a min-heap that only allows a maximum of `limit`
/// `Info` items to be added into it (removes smallest items when newer
/// items are added).
#[derive(Debug)]
pub struct LimitedHeap {
    pub limit: usize,
    heap: BinaryHeap<Info>,
}

#[allow(dead_code)]
impl LimitedHeap {
    pub fn new(limit: usize) -> LimitedHeap {
        LimitedHeap {
            limit,
            heap: BinaryHeap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn peek(&self) -> Option<&Info> {
        self.heap.peek()
    }

    pub fn pop(&mut self) -> Option<Info> {
        self.heap.pop()
    }

    pub fn push(&mut self, info: Info) {
        // if let Some(smallest) = self.peek() {
        //     if smallest.0 < info.0 {
        //         let _ = self.pop();
        //         self.heap.push(info);
        //     }
        // }

        self.heap.push(info);

        if self.heap.len() > self.limit {
            let _ = self.heap.pop();
        }
    }
}

// Displaying a `LimitedHeap` will print a list of each file and its size
// in descending size order.
impl Display for LimitedHeap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut items = vec![];

        let mut max_width = 0;
        for Info(bytes, path) in self.heap.clone().into_sorted_vec().into_iter() {
            let size = pretty_bytes::converter::convert(bytes as f64);
            max_width = max(size.len(), max_width);
            items.push((size, path))
        }

        let items = items
            .into_iter()
            .map(move |(size, path)| format!("{:<w$} {}", size, path.display(), w = max_width))
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", items)
    }
}
