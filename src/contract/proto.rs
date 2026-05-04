use super::{Contract, FuturesContract, FuturesMonth, FuturesRoot, YearFormat};

impl Contract {
    pub(crate) fn to_proto(self) -> crate::schema::Contract {
        match self {
            Self::Futures(contract) => crate::schema::Contract {
                value: Some(crate::schema::contract::Value::Futures(contract.to_proto())),
            },
        }
    }

    pub(crate) fn from_proto(proto: crate::schema::Contract) -> Result<Self, prost::DecodeError> {
        match proto.value {
            Some(crate::schema::contract::Value::Futures(contract)) => {
                Ok(Self::Futures(FuturesContract::from_proto(contract)?))
            }
            None => Err(prost::DecodeError::new("Missing contract variant")),
        }
    }
}

impl FuturesContract {
    fn to_proto(self) -> crate::schema::FuturesContract {
        crate::schema::FuturesContract {
            root: self.root.cme_code().to_string(),
            month: self.month.to_proto_i32(),
            year: u32::from(self.year),
            year_format: self.year_format.to_proto_i32(),
        }
    }

    fn from_proto(proto: crate::schema::FuturesContract) -> Result<Self, prost::DecodeError> {
        let root = FuturesRoot::new(&proto.root)
            .map_err(|err| prost::DecodeError::new(err.to_string()))?;
        let month = FuturesMonth::from_proto_i32(proto.month)?;
        let year = u16::try_from(proto.year)
            .map_err(|_| prost::DecodeError::new("Invalid futures contract year"))?;
        let year_format = YearFormat::from_proto_i32(proto.year_format)?;

        Ok(Self {
            root,
            month,
            year,
            year_format,
        })
    }
}

impl YearFormat {
    fn to_proto_i32(self) -> i32 {
        match self {
            Self::OneDigit => 1,
            Self::TwoDigit => 2,
        }
    }

    fn from_proto_i32(value: i32) -> Result<Self, prost::DecodeError> {
        match value {
            1 => Ok(Self::OneDigit),
            2 => Ok(Self::TwoDigit),
            _ => Err(prost::DecodeError::new(format!(
                "Invalid futures year format value: {value}",
            ))),
        }
    }
}

impl FuturesMonth {
    fn to_proto_i32(self) -> i32 {
        i32::from(self.number())
    }

    fn from_proto_i32(value: i32) -> Result<Self, prost::DecodeError> {
        match value {
            1 => Ok(Self::January),
            2 => Ok(Self::February),
            3 => Ok(Self::March),
            4 => Ok(Self::April),
            5 => Ok(Self::May),
            6 => Ok(Self::June),
            7 => Ok(Self::July),
            8 => Ok(Self::August),
            9 => Ok(Self::September),
            10 => Ok(Self::October),
            11 => Ok(Self::November),
            12 => Ok(Self::December),
            _ => Err(prost::DecodeError::new(format!(
                "Invalid futures month value: {value}",
            ))),
        }
    }
}
