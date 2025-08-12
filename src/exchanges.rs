use rust_decimal::Decimal;

#[derive(strum::Display, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExchangeName {
    Paradex,
    Hyperliquid,
    Kraken,
    Extended,
    Bebop,
    Avnu,
    Lmax,
}

#[derive(Debug, Clone)]
pub struct FillUpdate {
    pub size: Decimal,
    pub average_price: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrderStatus {
    New,
    Untriggered,
    Open,
    Closed,
    Cancelled,
    Rejected(String),
}