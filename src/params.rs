#[cfg(feature = "postgres")]
use crate::ArrayType;
use crate::Value;
#[cfg(feature = "postgres")]
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::Arguments;

#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    Null,
    Vector(Vec<Value>),
}

impl From<Vec<Value>> for Params {
    fn from(x: Vec<Value>) -> Params {
        Params::Vector(x)
    }
}

#[cfg(feature = "postgres")]
impl<'q> sqlx::IntoArguments<'q, sqlx::Postgres> for Params {
    fn into_arguments(self) -> sqlx::postgres::PgArguments {
        let mut args = sqlx::postgres::PgArguments::default();
        match self {
            Params::Null => {}
            Params::Vector(values) => {
                for value in values {
                    match value {
                        Value::Bool(v) => args.add(v),
                        Value::Tinyint(v) => args.add(v),
                        Value::Smallint(v) => args.add(v),
                        Value::Int(v) => args.add(v),
                        Value::Bigint(v) => args.add(v),
                        Value::TinyUnsigned(v) => {
                            args.add(v.map(|vv| vv as i8));
                        }
                        Value::SmallUnsigned(v) => {
                            args.add(v.map(|vv| vv as i16));
                        }
                        Value::Unsigned(v) => {
                            args.add(v.map(|vv| vv as i32));
                        }
                        Value::BigUnsigned(v) => {
                            args.add(v.map(|vv| vv as i64));
                        }
                        Value::Float(v) => args.add(v),
                        Value::Double(v) => args.add(v),
                        Value::BigDecimal(v) => args.add(v.as_deref()),
                        Value::Char(v) => {
                            args.add(v.map(|vv| vv.to_string()));
                        }
                        Value::String(v) => args.add(v),
                        Value::Bytes(v) => args.add(v.as_deref()),
                        Value::Json(v) => args.add(v.as_deref()),
                        Value::ChronoDate(v) => args.add(v.as_deref()),
                        Value::ChronoTime(v) => args.add(v.as_deref()),
                        Value::ChronoDateTime(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeUtc(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeLocal(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeWithTimeZone(v) => args.add(v.as_deref()),
                        // Value::Object(v) => args.add(v.as_deref()),
                        Value::Array(ty, v) => match ty {
                            ArrayType::Bool => {
                                let value: Option<Vec<bool>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Bool");
                                args.add(value)
                            }
                            ArrayType::Tinyint => {
                                let value: Option<Vec<i8>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::TinyInt");
                                args.add(value)
                            }
                            ArrayType::Smallint => {
                                let value: Option<Vec<i16>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::SmallInt");
                                args.add(value)
                            }
                            ArrayType::Int => {
                                let value: Option<Vec<i32>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Int");
                                args.add(value)
                            }
                            ArrayType::Bigint => {
                                let value: Option<Vec<i64>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::BigInt");
                                args.add(value)
                            }
                            ArrayType::TinyUnsigned => {
                                let value: Option<Vec<u8>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::TinyUnsigned",
                                );
                                let value: Option<Vec<i16>> =
                                    value.map(|vec| vec.into_iter().map(|i| i as i16).collect());
                                args.add(value)
                            }
                            ArrayType::SmallUnsigned => {
                                let value: Option<Vec<u16>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::SmallUnsigned",
                                );
                                let value: Option<Vec<i32>> =
                                    value.map(|vec| vec.into_iter().map(|i| i as i32).collect());
                                args.add(value)
                            }
                            ArrayType::Unsigned => {
                                let value: Option<Vec<u32>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Unsigned");
                                let value: Option<Vec<i64>> =
                                    value.map(|vec| vec.into_iter().map(|i| i as i64).collect());
                                args.add(value)
                            }
                            ArrayType::BigUnsigned => {
                                let value: Option<Vec<u64>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::BigUnsigned",
                                );
                                let value: Option<Vec<i64>> = value.map(|vec| {
                                    vec.into_iter()
                                        .map(|i| <i64 as TryFrom<u64>>::try_from(i).unwrap())
                                        .collect()
                                });
                                args.add(value)
                            }
                            ArrayType::Float => {
                                let value: Option<Vec<f32>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Float");
                                args.add(value)
                            }
                            ArrayType::Double => {
                                let value: Option<Vec<f64>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Double");
                                args.add(value)
                            }
                            ArrayType::String => {
                                let value: Option<Vec<String>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::String");
                                args.add(value)
                            }
                            ArrayType::Char => {
                                let value: Option<Vec<char>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Char");
                                let value: Option<Vec<String>> = value
                                    .map(|vec| vec.into_iter().map(|c| c.to_string()).collect());
                                args.add(value)
                            }
                            ArrayType::Bytes => {
                                let value: Option<Vec<Vec<u8>>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Bytes");
                                args.add(value)
                            }
                            ArrayType::ChronoDate => {
                                let value: Option<Vec<NaiveDate>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::ChronoDate",
                                );
                                args.add(value);
                            }
                            ArrayType::ChronoTime => {
                                let value: Option<Vec<NaiveTime>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::ChronoTime",
                                );
                                args.add(value);
                            }
                            ArrayType::ChronoDateTime => {
                                let value: Option<Vec<NaiveDateTime>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::ChronoDateTime",
                                );
                                args.add(value);
                            }
                            ArrayType::ChronoDateTimeUtc => {
                                let value: Option<Vec<DateTime<Utc>>> = Value::Array(ty, v).expect(
                                    "This Value::Array should consist of Value::ChronoDateTimeUtc",
                                );
                                args.add(value);
                            }
                            ArrayType::ChronoDateTimeLocal => {
                                let value: Option<Vec<DateTime<Local>>> = Value::Array(ty, v).expect(
                            "This Value::Array should consist of Value::ChronoDateTimeLocal",
                        );
                                args.add(value);
                            }
                            ArrayType::ChronoDateTimeWithTimeZone => {
                                let value: Option<Vec<DateTime<FixedOffset>>> = Value::Array(ty, v).expect(
                            "This Value::Array should consist of Value::ChronoDateTimeWithTimeZone",
                        );
                                args.add(value);
                            }

                            ArrayType::BigDecimal => {
                                let value: Option<Vec<sqlx::types::BigDecimal>> = Value::Array(
                                    ty, v,
                                )
                                .expect("This Value::Array should consist of Value::BigDecimal");
                                args.add(value);
                            }
                            ArrayType::Json => {
                                let value: Option<Vec<serde_json::Value>> = Value::Array(ty, v)
                                    .expect("This Value::Array should consist of Value::Json");
                                args.add(value);
                            }
                        },
                    }
                }
            }
        };
        args
    }
}

#[cfg(feature = "mysql")]
impl<'q> sqlx::IntoArguments<'q, sqlx::MySql> for Params {
    fn into_arguments(self) -> sqlx::mysql::MySqlArguments {
        let mut args = sqlx::mysql::MySqlArguments::default();
        match self {
            Params::Null => {}
            Params::Vector(values) => {
                for value in values {
                    match value {
                        Value::Bool(v) => args.add(v),
                        Value::Tinyint(v) => args.add(v),
                        Value::Smallint(v) => args.add(v),
                        Value::Int(v) => args.add(v),
                        Value::Bigint(v) => args.add(v),
                        Value::TinyUnsigned(v) => {
                            args.add(v.map(|vv| vv as i8));
                        }
                        Value::SmallUnsigned(v) => {
                            args.add(v.map(|vv| vv as i16));
                        }
                        Value::Unsigned(v) => {
                            args.add(v.map(|vv| vv as i32));
                        }
                        Value::BigUnsigned(v) => {
                            args.add(v.map(|vv| vv as i64));
                        }
                        Value::Float(v) => args.add(v),
                        Value::Double(v) => args.add(v),
                        Value::BigDecimal(v) => args.add(v.as_deref()),
                        Value::Char(v) => {
                            args.add(v.map(|vv| vv.to_string()));
                        }
                        Value::String(v) => args.add(v),
                        Value::Bytes(v) => args.add(v.as_deref()),
                        Value::Json(v) => args.add(v.as_deref()),
                        Value::ChronoDate(v) => args.add(v.as_deref()),
                        Value::ChronoTime(v) => args.add(v.as_deref()),
                        Value::ChronoDateTime(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeUtc(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeLocal(v) => args.add(v.as_deref()),
                        Value::ChronoDateTimeWithTimeZone(v) => {
                            args.add(
                                Value::ChronoDateTimeWithTimeZone(v)
                                    .chrono_as_naive_utc_in_string(),
                            );
                        }
                        Value::Array(_, _) => {
                            panic!("Mysql doesn't support array arguments");
                        }
                    }
                }
            }
        };

        args
    }
}

#[cfg(feature = "sqlite")]
impl<'q> sqlx::IntoArguments<'q, sqlx::Sqlite> for Params {
    fn into_arguments(self) -> sqlx::sqlite::SqliteArguments<'q> {
        let mut args = sqlx::sqlite::SqliteArguments::default();
        match self {
            Params::Null => {}
            Params::Vector(values) => {
                for value in values {
                    match value {
                        Value::Bool(v) => args.add(v),
                        Value::Tinyint(v) => args.add(v),
                        Value::Smallint(v) => args.add(v),
                        Value::Int(v) => args.add(v),
                        Value::Bigint(v) => args.add(v),
                        Value::TinyUnsigned(v) => {
                            args.add(v.map(|vv| vv as i8));
                        }
                        Value::SmallUnsigned(v) => {
                            args.add(v.map(|vv| vv as i16));
                        }
                        Value::Unsigned(v) => {
                            args.add(v.map(|vv| vv as i32));
                        }
                        Value::BigUnsigned(v) => {
                            args.add(v.map(|vv| vv as i64));
                        }
                        Value::Float(v) => args.add(v),
                        Value::Double(v) => args.add(v),
                        Value::BigDecimal(v) => {
                            use bigdecimal::ToPrimitive;
                            args.add(
                                v.map(|d| d.to_f64().expect("Fail to convert bigdecimal as f64")),
                            );
                        }
                        Value::Char(v) => {
                            args.add(v.map(|vv| vv.to_string()));
                        }
                        Value::String(v) => args.add(v),
                        Value::Bytes(v) => args.add(v.map(|t| *t)),
                        Value::Json(v) => args.add(v.map(|t| *t)),
                        Value::ChronoDate(v) => args.add(v.map(|t| *t)),
                        Value::ChronoTime(v) => args.add(v.map(|t| *t)),
                        Value::ChronoDateTime(v) => args.add(v.map(|t| *t)),
                        Value::ChronoDateTimeUtc(v) => args.add(v.map(|t| *t)),
                        Value::ChronoDateTimeLocal(v) => args.add(v.map(|t| *t)),
                        Value::ChronoDateTimeWithTimeZone(v) => {
                            args.add(
                                Value::ChronoDateTimeWithTimeZone(v)
                                    .chrono_as_naive_utc_in_string(),
                            );
                        }
                        // Value::Object(v) => args.add(v.as_deref()),
                        Value::Array(_ty, _v) => {
                            panic!("Mysql doesn't support array arguments");
                        }
                    }
                }
            }
        };

        args
    }
}
