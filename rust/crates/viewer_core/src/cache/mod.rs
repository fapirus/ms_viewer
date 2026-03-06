use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachePolicy {
    pub render_window: usize,
    pub prefetch_radius: usize,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            render_window: 3,
            prefetch_radius: 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct LruPageCache {
    order: VecDeque<u32>,
    capacity: usize,
}

impl LruPageCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            order: VecDeque::new(),
            capacity,
        }
    }

    pub fn touch(&mut self, page_index: u32) {
        if let Some(position) = self.order.iter().position(|value| *value == page_index) {
            self.order.remove(position);
        }
        self.order.push_front(page_index);

        while self.order.len() > self.capacity {
            let _ = self.order.pop_back();
        }
    }

    pub fn pages(&self) -> Vec<u32> {
        self.order.iter().copied().collect()
    }
}

pub fn compute_prefetch_window(current: u32, max_index: u32, radius: usize) -> Vec<u32> {
    let start = current.saturating_sub(radius as u32);
    let end = (current + radius as u32).min(max_index);
    (start..=end).collect()
}
