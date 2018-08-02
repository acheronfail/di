use ansi_term::Colour;
use pretty_bytes;
use std::cmp::{max, Ordering};
use std::collections::BinaryHeap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Info(pub u64, pub PathBuf);

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

impl LimitedHeap {
    pub fn new(limit: usize) -> LimitedHeap {
        LimitedHeap {
            limit,
            heap: BinaryHeap::new(),
        }
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
        let len = self.len();
        if len > 0 && len == self.limit {
            if self.peek().unwrap().0 < info.0 {
                let _ = self.pop();
                self.heap.push(info);
            }
        } else {
            self.heap.push(info);
        }
    }
}

// Displaying a `LimitedHeap` will print a list of each file and its size in
// descending order.
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
            .map(move |(size, path)| {
                let path_str = Colour::Fixed(244).paint(format!("{}", path.display()));
                let size_str = format!("{:>w$}", size, w = max_width);
                format!(" {}  {}", Colour::Yellow.paint(size_str), path_str)
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", items)
    }
}
