use chrono::Datelike;

use super::{FuturesContract, FuturesContractParseError, FuturesMonth, FuturesRoot};

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

        let root = root.trim();
        if root.is_empty() {
            return Err(FuturesContractParseError::InvalidRoot(
                activ_symbol.to_string(),
            ));
        }
        if year_month.len() < 2 {
            return Err(FuturesContractParseError::MissingMonthCode);
        }

        let (year_str, month_char) = split_last_char(year_month.trim())?;

        let root = FuturesRoot::new(root)?;
        let month = FuturesMonth::from_code(month_char)
            .ok_or(FuturesContractParseError::InvalidMonthCode(month_char))?;
        let year = normalize_year(year_str)?;

        Ok(FuturesContract { root, month, year })
    }

    pub fn build(self) -> Result<FuturesContract, FuturesContractParseError> {
        let raw_symbol = self.raw_symbol.trim();
        if raw_symbol.is_empty() {
            return Err(FuturesContractParseError::Empty);
        }

        let (root, month_char, year_str) = split_raw_contract_symbol(raw_symbol)?;

        let root = FuturesRoot::new(root)?;
        let month = FuturesMonth::from_code(month_char)
            .ok_or(FuturesContractParseError::InvalidMonthCode(month_char))?;
        let year = normalize_year(year_str)?;

        Ok(FuturesContract { root, month, year })
    }
}

fn split_last_char(value: &str) -> Result<(&str, char), FuturesContractParseError> {
    value
        .char_indices()
        .last()
        .map(|(index, character)| (&value[..index], character))
        .ok_or(FuturesContractParseError::MissingMonthCode)
}

fn split_raw_contract_symbol(
    raw_symbol: &str,
) -> Result<(&str, char, &str), FuturesContractParseError> {
    let (month_pos, month_char) = raw_symbol
        .char_indices()
        .rfind(|(_, c)| c.is_ascii_alphabetic())
        .ok_or(FuturesContractParseError::MissingMonthCode)?;
    let (root, month_and_year) = raw_symbol.split_at(month_pos);
    let (_, year_str) = month_and_year.split_at(month_char.len_utf8());

    if year_str.is_empty() {
        return Err(FuturesContractParseError::MissingYear);
    }
    if !year_str.bytes().all(|c| c.is_ascii_digit()) {
        return Err(FuturesContractParseError::InvalidYear(year_str.to_string()));
    }

    Ok((root, month_char, year_str))
}

fn normalize_year(year_str: &str) -> Result<u16, FuturesContractParseError> {
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
            if full_year < current_year {
                full_year += 10;
            }
            Ok(full_year)
        }
        2 => Ok(2000 + year_short),
        _ => Err(FuturesContractParseError::InvalidYear(year_str.to_string())),
    }
}
