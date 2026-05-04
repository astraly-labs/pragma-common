use chrono::Datelike;

use super::{FuturesContract, FuturesContractParseError, FuturesMonth, FuturesRoot, YearFormat};

#[derive(Debug, Clone, Copy)]
pub struct FuturesContractBuilder<'a> {
    raw_symbol: &'a str,
}

impl<'a> FuturesContractBuilder<'a> {
    pub const fn raw(raw_symbol: &'a str) -> Self {
        Self { raw_symbol }
    }

    /// Parse an ACTIV-format contract string: `ROOT/YYMONTHCODE` (e.g. `GC/26M`).
    pub fn activ(activ_symbol: &str) -> Result<FuturesContract, FuturesContractParseError> {
        let (root, year_month) = activ_symbol
            .split_once('/')
            .ok_or_else(|| FuturesContractParseError::InvalidRoot(activ_symbol.to_string()))?;

        let root = root.trim().to_ascii_uppercase();
        if root.is_empty() {
            return Err(FuturesContractParseError::InvalidRoot(
                activ_symbol.to_string(),
            ));
        }
        if year_month.len() < 2 {
            return Err(FuturesContractParseError::MissingMonthCode);
        }

        let month_char = year_month
            .chars()
            .last()
            .ok_or(FuturesContractParseError::MissingMonthCode)?;
        let year_str = &year_month[..year_month.len() - 1];

        let root = FuturesRoot::new(&root)?;
        let month = FuturesMonth::from_code(month_char)
            .ok_or(FuturesContractParseError::InvalidMonthCode(month_char))?;
        let (year, year_format) = normalize_year(year_str)?;

        Ok(FuturesContract {
            root,
            month,
            year,
            year_format,
        })
    }

    pub fn build(self) -> Result<FuturesContract, FuturesContractParseError> {
        let normalized = self.raw_symbol.trim().to_ascii_uppercase();
        if normalized.is_empty() {
            return Err(FuturesContractParseError::Empty);
        }

        let month_pos = normalized
            .rfind(|c: char| c.is_ascii_alphabetic())
            .ok_or(FuturesContractParseError::MissingMonthCode)?;
        let root = &normalized[..month_pos];
        let month_char = normalized
            .chars()
            .nth(month_pos)
            .ok_or(FuturesContractParseError::MissingMonthCode)?;
        let year_str = &normalized[month_pos + 1..];

        if year_str.is_empty() {
            return Err(FuturesContractParseError::MissingYear);
        }
        if !year_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(FuturesContractParseError::InvalidYear(year_str.to_string()));
        }

        let root = FuturesRoot::new(root)?;
        let month = FuturesMonth::from_code(month_char)
            .ok_or(FuturesContractParseError::InvalidMonthCode(month_char))?;
        let (year, year_format) = normalize_year(year_str)?;

        Ok(FuturesContract {
            root,
            month,
            year,
            year_format,
        })
    }
}

fn normalize_year(year_str: &str) -> Result<(u16, YearFormat), FuturesContractParseError> {
    let year_short: u16 = year_str
        .parse()
        .map_err(|_| FuturesContractParseError::InvalidYear(year_str.to_string()))?;

    match year_str.len() {
        1 => {
            let current_year = chrono::Utc::now().date_naive().year();
            let current_year = u16::try_from(current_year)
                .map_err(|_| FuturesContractParseError::InvalidYear(year_str.to_string()))?;
            let decade = (current_year / 10) * 10;
            let mut full_year = decade + year_short;
            if full_year + 1 < current_year {
                full_year += 10;
            }
            Ok((full_year, YearFormat::OneDigit))
        }
        2 => Ok((2000 + year_short, YearFormat::TwoDigit)),
        _ => Err(FuturesContractParseError::InvalidYear(year_str.to_string())),
    }
}
