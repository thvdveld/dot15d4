use crate::frame::Address;

use heapless::Vec;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) struct Neighbor {
    address: Address,
    stats: NeighborStats,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) struct NeighborStats {
    rx_count: usize,
    join_priority: u8,
}

#[derive(Default)]
pub(super) struct NeighborTable<const N: usize> {
    neighbors: Vec<Neighbor, N>,
}

impl<const N: usize> NeighborTable<N> {
    pub fn add(&mut self, neighbor: Neighbor) {
        _ = self.neighbors.push(neighbor);
    }

    pub fn remove(&mut self, address: Address) {
        self.neighbors.retain(|n| n.address != address);
    }

    pub fn get(&self, address: Address) -> Option<&Neighbor> {
        self.neighbors.iter().find(|n| n.address == address)
    }

    pub fn get_mut(&mut self, address: Address) -> Option<&mut Neighbor> {
        self.neighbors.iter_mut().find(|n| n.address == address)
    }

    pub fn len(&self) -> usize {
        self.neighbors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }

    pub fn clear(&mut self) {
        self.neighbors.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Neighbor> {
        self.neighbors.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Neighbor> {
        self.neighbors.iter_mut()
    }

    /// Return the best time source based on the amount of Enhanced Beacons we received and the
    /// join priority of the neighbor.
    pub fn get_best_time_source(&self, current: NeighborStats) -> Option<&Neighbor> {
        self.neighbors.iter().find(|n| {
            n.stats.rx_count > current.rx_count / 2 && n.stats.join_priority < current.join_priority
        })
    }
}
