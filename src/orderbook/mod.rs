pub mod depth;
pub mod snapshot;
pub mod update;

pub use depth::*;
pub use snapshot::*;
#[allow(unused)]
pub use update::*;

use std::collections::BTreeMap;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};

#[derive(Debug, thiserror::Error)]
pub enum OrderbookError {
    #[error("Received invalid bid")]
    InvalidBid,
    #[error("Received invalid ask")]
    InvalidAsk,
}

/// A orderbook order book that handles updates and snapshots.
///
/// This struct wraps the core `InnerOrderbook` and manages pending updates before a snapshot is applied.
#[derive(Debug, Clone)]
pub struct Orderbook {
    /// The core order book with bids and asks
    orderbook: InnerOrderbook,
    /// Tracks whether a snapshot has been applied
    snapshot_applied: bool,
    /// Pending updates before snapshot
    pending_updates: Option<OrderbookUpdate>,
}

impl Orderbook {
    /// Create a new orderbook with the given depth.
    pub fn new(depth: usize) -> Self {
        Self {
            orderbook: InnerOrderbook {
                depth,
                bids: BTreeMap::new(),
                asks: BTreeMap::new(),
                last_update_id: 0,
            },
            snapshot_applied: false,
            pending_updates: None,
        }
    }

    /// Returns the last update id applied to the current orderbook.
    pub const fn last_update_id(&self) -> u64 {
        self.orderbook.last_update_id()
    }

    /// Returns the next update id & increment the state of the inner `Orderbook`.
    pub const fn next_update_id(&self) -> u64 {
        self.orderbook.next_update_id()
    }

    /// Returns true if a snapshot has been applied to the current orderbook, else false.
    pub const fn snapshot_was_applied(&self) -> bool {
        self.snapshot_applied
    }

    /// Returns the current `best_bid` - if any.
    pub fn best_bid(&self) -> Option<&BigDecimal> {
        self.orderbook.best_bid()
    }

    /// Returns the current `best_ask` - if any.
    pub fn best_ask(&self) -> Option<&BigDecimal> {
        self.orderbook.best_ask()
    }

    /// Get the mid price (average of best bid and ask).
    pub fn mid_price(&self) -> Option<BigDecimal> {
        self.orderbook.mid_price()
    }

    /// Compute depth at a given percentage.
    pub fn depth(&self, percentage: f64) -> Option<DepthLevel> {
        self.orderbook.depth(percentage)
    }

    /// Applies an update to the order book.
    ///
    /// Before a snapshot, updates are merged into `pending_updates`. After a snapshot, updates are
    /// applied directly if their `update_id` is newer than the current `last_update_id`.
    pub fn apply_update(
        &mut self,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        update_id: u64,
    ) -> Result<(), OrderbookError> {
        let bids_bd: Vec<(BigDecimal, f64)> = bids
            .into_iter()
            .map(|(p, q)| BigDecimal::from_f64(p).map(|bd| (bd, q)))
            .collect::<Option<_>>()
            .ok_or(OrderbookError::InvalidAsk)?;

        let asks_bd: Vec<(BigDecimal, f64)> = asks
            .into_iter()
            .map(|(p, q)| BigDecimal::from_f64(p).map(|bd| (bd, q)))
            .collect::<Option<_>>()
            .ok_or(OrderbookError::InvalidBid)?;

        if self.snapshot_applied {
            // After snapshot, apply only if update_id is newer
            if update_id > self.orderbook.last_update_id {
                self.orderbook.apply_update(bids_bd, asks_bd, update_id);
            }
        } else {
            // Before snapshot, merge into pending updates
            if let Some(pending) = self.pending_updates.as_mut() {
                pending.merge(bids_bd, asks_bd, update_id);
            } else {
                self.pending_updates = Some(OrderbookUpdate::new(bids_bd, asks_bd, update_id));
            }
        }

        Ok(())
    }

    /// Applies a snapshot, replacing the current order book state.
    ///
    /// After applying the snapshot, any pending updates with a higher `update_id` are applied.
    pub fn apply_snapshot(
        &mut self,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        update_id: u64,
    ) -> Result<(), OrderbookError> {
        // Convert f64 prices to BigDecimal for precision
        let bids_bd: Vec<(BigDecimal, f64)> = bids
            .into_iter()
            .map(|(p, q)| BigDecimal::from_f64(p).map(|bd| (bd, q)))
            .collect::<Option<_>>()
            .ok_or(OrderbookError::InvalidBid)?;
        let asks_bd: Vec<(BigDecimal, f64)> = asks
            .into_iter()
            .map(|(p, q)| BigDecimal::from_f64(p).map(|bd| (bd, q)))
            .collect::<Option<_>>()
            .ok_or(OrderbookError::InvalidAsk)?;

        self.orderbook
            .clear_and_apply_snapshot(bids_bd, asks_bd, update_id);
        self.apply_pending_updates_after_snapshot();
        self.snapshot_applied = true;
        Ok(())
    }

    /// Applies pending updates after a snapshot, filtering by update ID.
    fn apply_pending_updates_after_snapshot(&mut self) {
        // No pending updates to process
        let Some(pending) = self.pending_updates.take() else {
            return;
        };

        // The pending updates are too old - irrelevant
        if pending.latest_update_id <= self.orderbook.last_update_id {
            return;
        }

        let bids: Vec<(BigDecimal, f64)> = pending
            .bids
            .into_iter()
            .filter_map(|(price, (qty, uid))| {
                if uid > self.orderbook.last_update_id {
                    Some((price, qty))
                } else {
                    None
                }
            })
            .collect();

        let asks: Vec<(BigDecimal, f64)> = pending
            .asks
            .into_iter()
            .filter_map(|(price, (qty, uid))| {
                if uid > self.orderbook.last_update_id {
                    Some((price, qty))
                } else {
                    None
                }
            })
            .collect();

        if !bids.is_empty() || !asks.is_empty() {
            self.orderbook
                .apply_update(bids, asks, pending.latest_update_id);
        }
    }
}

/// Represents a merged order book update with per-level update IDs.
#[derive(Debug, Clone)]
struct OrderbookUpdate {
    bids: BTreeMap<BigDecimal, (f64, u64)>, // price -> (quantity, update_id)
    asks: BTreeMap<BigDecimal, (f64, u64)>, // price -> (quantity, update_id)
    latest_update_id: u64,
}

impl OrderbookUpdate {
    /// Creates a new update from bids, asks, and an update ID.
    fn new(bids: Vec<(BigDecimal, f64)>, asks: Vec<(BigDecimal, f64)>, update_id: u64) -> Self {
        let mut bids_map = BTreeMap::new();
        let mut asks_map = BTreeMap::new();

        for (price, qty) in bids {
            if qty > 0.0 {
                bids_map.insert(price, (qty, update_id));
            }
        }
        for (price, qty) in asks {
            if qty > 0.0 {
                asks_map.insert(price, (qty, update_id));
            }
        }

        Self {
            bids: bids_map,
            asks: asks_map,
            latest_update_id: update_id,
        }
    }

    /// Merges a new update into this one, keeping the latest data per price level.
    ///
    /// Updates are only applied if the new `update_id` is greater than or equal to the existing one.
    fn merge(
        &mut self,
        bids: Vec<(BigDecimal, f64)>,
        asks: Vec<(BigDecimal, f64)>,
        update_id: u64,
    ) {
        for (price, qty) in bids {
            if qty == 0.0 {
                self.bids.remove(&price);
            } else {
                self.bids
                    .entry(price)
                    .and_modify(|(existing_qty, existing_id)| {
                        if update_id > *existing_id {
                            *existing_qty = qty;
                            *existing_id = update_id;
                        }
                    })
                    .or_insert((qty, update_id));
            }
        }
        for (price, qty) in asks {
            if qty == 0.0 {
                self.asks.remove(&price);
            } else {
                self.asks
                    .entry(price)
                    .and_modify(|(existing_qty, existing_id)| {
                        if update_id > *existing_id {
                            *existing_qty = qty;
                            *existing_id = update_id;
                        }
                    })
                    .or_insert((qty, update_id));
            }
        }
        self.latest_update_id = self.latest_update_id.max(update_id);
    }
}

/// The core order book structure.
#[derive(Debug, Clone)]
struct InnerOrderbook {
    depth: usize,                    // Max depth level of the orderbook
    bids: BTreeMap<BigDecimal, f64>, // price -> quantity
    asks: BTreeMap<BigDecimal, f64>, // price -> quantity
    last_update_id: u64,
}

impl InnerOrderbook {
    const fn last_update_id(&self) -> u64 {
        self.last_update_id
    }

    /// Returns the next update id & increment the state.
    const fn next_update_id(&self) -> u64 {
        self.last_update_id + 1
    }

    /// Clear the current orderbook.
    fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }

    /// Clear the current orderbook & replace it with a new snapshot.
    fn clear_and_apply_snapshot(
        &mut self,
        bids: Vec<(BigDecimal, f64)>,
        asks: Vec<(BigDecimal, f64)>,
        update_id: u64,
    ) {
        self.clear();
        for (price, qty) in bids {
            if qty > 0.0 {
                self.bids.insert(price, qty);
            }
        }
        for (price, qty) in asks {
            if qty > 0.0 {
                self.asks.insert(price, qty);
            }
        }
        self.last_update_id = update_id;
    }

    fn apply_update(
        &mut self,
        bids: Vec<(BigDecimal, f64)>,
        asks: Vec<(BigDecimal, f64)>,
        update_id: u64,
    ) {
        for (price, qty) in bids {
            if qty.is_zero() {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, qty);
            }
        }
        for (price, qty) in asks {
            if qty.is_zero() {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, qty);
            }
        }
        self.last_update_id = update_id;
        self.truncate_to_depth();
    }

    fn best_bid(&self) -> Option<&BigDecimal> {
        self.bids.iter().next_back().map(|(p, _)| p)
    }

    fn best_ask(&self) -> Option<&BigDecimal> {
        self.asks.iter().next().map(|(p, _)| p)
    }

    fn mid_price(&self) -> Option<BigDecimal> {
        let best_bid = self.best_bid();
        let best_ask = self.best_ask();
        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / BigDecimal::from(2)),
            _ => None,
        }
    }

    fn depth(&self, percentage: f64) -> Option<DepthLevel> {
        let mid = self.mid_price()?;
        let mid_price = mid.to_f64()?;

        let percentage_bd = BigDecimal::from_f64(percentage)?;
        let lower = &mid * (BigDecimal::from(1) - &percentage_bd);
        let upper = &mid * (BigDecimal::from(1) + &percentage_bd);

        let bid_depth: f64 = self.bids.range(&lower..=&mid).map(|(_, &qty)| qty).sum();
        let ask_depth: f64 = self.asks.range(&mid..=&upper).map(|(_, &qty)| qty).sum();

        Some(DepthLevel {
            bid: bid_depth * mid_price,
            ask: ask_depth * mid_price,
            percentage,
        })
    }

    fn truncate_to_depth(&mut self) {
        while self.bids.len() > self.depth {
            self.bids.pop_first();
        }

        while self.asks.len() > self.depth {
            self.asks.pop_last();
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orderbook() {
        let ob = Orderbook::new(10);
        assert_eq!(ob.last_update_id(), 0);
        assert!(!ob.snapshot_was_applied());
        assert!(ob.pending_updates.is_none());
        assert!(ob.mid_price().is_none());
    }

    #[test]
    fn test_apply_update_before_snapshot() {
        let mut ob = Orderbook::new(10);
        ob.apply_update(vec![(100.0, 10.0), (99.0, 5.0)], vec![(101.0, 15.0)], 1)
            .unwrap();

        let pending = ob.pending_updates.as_ref().unwrap();
        assert_eq!(pending.bids.len(), 2);
        assert_eq!(pending.asks.len(), 1);
        assert_eq!(pending.latest_update_id, 1);
        assert_eq!(pending.bids.get(&BigDecimal::from(100)), Some(&(10.0, 1)));
        assert_eq!(pending.bids.get(&BigDecimal::from(99)), Some(&(5.0, 1)));
        assert_eq!(pending.asks.get(&BigDecimal::from(101)), Some(&(15.0, 1)));
        assert_eq!(ob.orderbook.bids.len(), 0); // Not applied yet
    }

    #[test]
    fn test_merge_updates_before_snapshot() {
        let mut ob = Orderbook::new(10);
        ob.apply_update(vec![(100.0, 10.0)], vec![(101.0, 15.0)], 1)
            .unwrap();
        ob.apply_update(
            vec![(100.0, 0.0), (99.0, 5.0)], // Remove 100.0, add 99.0
            vec![(101.0, 20.0)],             // Update 101.0
            2,
        )
        .unwrap();

        let pending = ob.pending_updates.as_ref().unwrap();
        assert_eq!(pending.bids.len(), 1); // 100.0 removed
        assert_eq!(pending.asks.len(), 1);
        assert_eq!(pending.latest_update_id, 2);
        assert_eq!(pending.bids.get(&BigDecimal::from(99)), Some(&(5.0, 2)));
        assert_eq!(pending.asks.get(&BigDecimal::from(101)), Some(&(20.0, 2)));
    }

    #[test]
    fn test_apply_snapshot_no_pending() {
        let mut ob = Orderbook::new(10);
        ob.apply_snapshot(vec![(100.0, 10.0), (99.0, 5.0)], vec![(101.0, 15.0)], 5)
            .unwrap();

        assert!(ob.snapshot_was_applied());
        assert_eq!(ob.last_update_id(), 5);
        assert!(ob.pending_updates.is_none());
        assert_eq!(ob.orderbook.bids.len(), 2);
        assert_eq!(ob.orderbook.asks.len(), 1);
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(100)), Some(&10.0));
        assert_eq!(ob.mid_price(), Some(BigDecimal::from_f64(100.5).unwrap()));
    }

    #[test]
    fn test_apply_snapshot_with_pending() {
        let mut ob = Orderbook::new(10);
        ob.apply_update(vec![(100.0, 10.0)], vec![(102.0, 20.0)], 1)
            .unwrap();
        ob.apply_update(vec![(99.0, 5.0)], vec![(103.0, 15.0)], 3)
            .unwrap();
        ob.apply_snapshot(vec![(100.0, 8.0)], vec![(101.0, 12.0)], 2)
            .unwrap();

        assert_eq!(ob.last_update_id(), 3);
        assert_eq!(ob.orderbook.bids.len(), 2); // 100.0 from snapshot, 99.0 from update 3
        assert_eq!(ob.orderbook.asks.len(), 2); // 101.0 from snapshot, 103.0 from update 3
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(100)), Some(&8.0));
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(99)), Some(&5.0));
        assert_eq!(ob.orderbook.asks.get(&BigDecimal::from(102)), None); // Update 1 ignored
    }

    #[test]
    fn test_update_after_snapshot() {
        let mut ob = Orderbook::new(10);
        ob.apply_snapshot(vec![(100.0, 10.0)], vec![(101.0, 15.0)], 5)
            .unwrap();
        ob.apply_update(vec![(100.0, 0.0), (99.0, 5.0)], vec![(102.0, 20.0)], 6)
            .unwrap();

        assert_eq!(ob.last_update_id(), 6);
        assert_eq!(ob.orderbook.bids.len(), 1); // 100.0 removed
        assert_eq!(ob.orderbook.asks.len(), 2);
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(99)), Some(&5.0));
        assert_eq!(ob.orderbook.asks.get(&BigDecimal::from(102)), Some(&20.0));
    }

    #[test]
    fn test_old_update_after_snapshot() {
        let mut ob = Orderbook::new(10);
        ob.apply_snapshot(vec![(100.0, 10.0)], vec![(101.0, 15.0)], 5)
            .unwrap();
        ob.apply_update(vec![(100.0, 5.0)], vec![(101.0, 10.0)], 4)
            .unwrap();

        assert_eq!(ob.last_update_id(), 5); // Update 4 ignored
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(100)), Some(&10.0));
        assert_eq!(ob.orderbook.asks.get(&BigDecimal::from(101)), Some(&15.0));
    }

    #[test]
    fn test_invalid_price() {
        let mut ob = Orderbook::new(10);
        let result = ob.apply_update(vec![(f64::NAN, 10.0)], vec![(101.0, 15.0)], 1);
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_mid_price_and_depth() {
        let mut ob = Orderbook::new(10);
        ob.apply_snapshot(
            vec![(99.0, 5.0), (100.0, 10.0)],
            vec![(101.0, 15.0), (102.0, 20.0)],
            1,
        )
        .unwrap();

        let mid = ob.mid_price().unwrap();
        assert_eq!(mid, BigDecimal::from_f64(100.5).unwrap());
    }

    #[test]
    fn test_zero_quantity_in_snapshot() {
        let mut ob = Orderbook::new(10);
        ob.apply_snapshot(vec![(100.0, 0.0), (99.0, 5.0)], vec![(101.0, 0.0)], 1)
            .unwrap();

        assert_eq!(ob.orderbook.bids.len(), 1); // 100.0 ignored
        assert_eq!(ob.orderbook.asks.len(), 0); // 101.0 ignored
        assert_eq!(ob.orderbook.bids.get(&BigDecimal::from(99)), Some(&5.0));
    }
}
