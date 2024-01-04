use std::collections::HashMap;

use crate::{add_condition, OrderByType, Platform};
use crate::{Column, NotU8, Segment, SegmentType, Segments, Value, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct Wrapper {
    /// set 字段
    pub column_set: Vec<(String, Value)>,
    /// SQL查询字段
    pub sql_select: Option<String>,
    /// SQL起始语句
    pub sql_first: Option<String>,
    /// SQL结束语句
    pub sql_last: Option<String>,
    /// expression
    pub expression: Segments,
    /// 层级
    pub bracket_num: i32,
}

impl Wrapper {
    pub fn new() -> Self {
        Self::with_bracket_num(0)
    }

    pub fn with_bracket_num(bracket_num: i32) -> Self {
        Self {
            column_set: Vec::new(),
            sql_select: None,
            sql_first: None,
            sql_last: None,
            expression: Segments::with_bracket_num(bracket_num),
            bracket_num: bracket_num,
        }
    }

    pub fn get_sql(&self) -> (String, HashMap<String, Value>) {
        let (sql, params) = self.expression.get_sql();
        (sql, params)
    }

    pub fn build(&self, platform: &Platform, mut index: usize) -> (String, Vec<Value>) {
        let (sql, params) = self.get_sql();
        let sqls = sql.split_whitespace().collect::<Vec<_>>();
        let mut values = Vec::new();
        let mut build_sql = String::new();
        for s in sqls.into_iter() {
            build_sql.push_str(" ");
            if s.starts_with(":") {
                let val = params.get(&s[1..]).expect("param not found");
                if matches!(val, Value::Array(_, _)) {
                    let array = val.get_array();
                    for (i, arr) in array.iter().enumerate() {
                        if i != 0 {
                            build_sql.push_str(", ");
                        }
                        build_sql.push_str(&platform.mark(index));
                        values.push(arr.clone());
                        index += 1;
                    }
                } else {
                    values.push(val.clone());
                    build_sql.push_str(&platform.mark(index));
                    index += 1;
                }
            } else {
                build_sql.push_str(s);
            }
        }
        if build_sql.trim().is_empty() {
            build_sql = " 1 = 1 ".to_string();
        }
        (build_sql, values)
    }
}

impl Wrapper {
    pub fn do_it(mut self, condition: bool, segment_type: SegmentType, segment: Segment) -> Self {
        if condition {
            self.expression.add(segment_type, segment);
        }
        self
    }

    pub fn sql_first(mut self, sql: &str) -> Self {
        self.sql_first = Some(sql.to_string());
        self
    }

    pub fn sql_last(mut self, sql: &str) -> Self {
        self.sql_last = Some(sql.to_string());
        self
    }

    pub fn or(self) -> Self {
        self.do_it(true, SegmentType::Normal, Segment::Or)
    }

    fn add_nested_condition<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self {
        let bracket_num = self.bracket_num + 1;
        if condition {
            let instance = f(Self::with_bracket_num(bracket_num));
            self.do_it(
                true,
                SegmentType::Normal,
                Segment::Bracket(Box::new(instance)),
            )
        } else {
            self
        }
    }

    pub fn and_fn<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        self.add_nested_condition(true, f)
    }

    pub fn or_fn<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        self.or().add_nested_condition(true, f)
    }

    pub fn eq<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Eq)
    }

    pub fn ne<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Ne)
    }

    pub fn gt<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Gt)
    }

    pub fn ge<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Ge)
    }

    pub fn lt<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Lt)
    }

    pub fn le<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Le)
    }

    pub fn like<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, Like)
    }

    pub fn like_left<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, LikeLeft)
    }

    pub fn like_right<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, LikeRight)
    }

    pub fn not_like<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, NotLike)
    }

    pub fn not_like_right<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, NotLikeRight)
    }

    pub fn not_like_left<C, V>(self, column: C, val: V) -> Self
    where
        C: Column,
        V: Into<Value>,
    {
        add_condition!(self, column, val, NotLikeLeft)
    }

    pub fn in_list<C, V>(self, column: C, val: Vec<V>) -> Self
    where
        C: Column,
        V: Into<Value> + NotU8 + ValueType,
    {
        self.in_condition(true, column, val)
    }

    pub fn in_condition<C, V>(self, condition: bool, column: C, val: Vec<V>) -> Self
    where
        C: Column,
        V: Into<Value> + NotU8 + ValueType,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(condition, SegmentType::Normal, Segment::In(c, val.into()))
    }

    pub fn not_in<C, V>(self, column: C, val: Vec<V>) -> Self
    where
        C: Column,
        V: Into<Value> + NotU8 + ValueType,
    {
        self.not_in_condition(true, column, val)
    }

    pub fn not_in_condition<C, V>(self, condition: bool, column: C, val: Vec<V>) -> Self
    where
        C: Column,
        V: Into<Value> + NotU8 + ValueType,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(
            condition,
            SegmentType::Normal,
            Segment::NotIn(c, val.into()),
        )
    }

    pub fn between<C, V>(self, column: C, val1: V, val2: V) -> Self
    where
        C: Column,
        V: Into<Value> + ValueType,
    {
        self.between_condition(true, column, val1, val2)
    }

    pub fn not_between<C, V>(self, column: C, val1: V, val2: V) -> Self
    where
        C: Column,
        V: Into<Value> + ValueType,
    {
        self.not_between_condition(true, column, val1, val2)
    }

    pub fn between_condition<C, V>(self, condition: bool, column: C, val1: V, val2: V) -> Self
    where
        C: Column,
        V: Into<Value> + ValueType,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(
            condition,
            SegmentType::Normal,
            Segment::Between(c, val1.into(), val2.into()),
        )
    }

    pub fn not_between_condition<C, V>(self, condition: bool, column: C, val1: V, val2: V) -> Self
    where
        C: Column,
        V: Into<Value> + ValueType,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(
            condition,
            SegmentType::Normal,
            Segment::NotBetween(c, val1.into(), val2.into()),
        )
    }

    pub fn is_null<C>(self, column: C) -> Self
    where
        C: Column,
    {
        self.is_null_condition(true, column)
    }

    pub fn is_not_null<C>(self, column: C) -> Self
    where
        C: Column,
    {
        self.is_not_null_condition(true, column)
    }

    pub fn is_null_condition<C>(self, condition: bool, column: C) -> Self
    where
        C: Column,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(condition, SegmentType::Normal, Segment::IsNull(c))
    }

    pub fn is_not_null_condition<C>(self, condition: bool, column: C) -> Self
    where
        C: Column,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(condition, SegmentType::Normal, Segment::IsNotNull(c))
    }

    pub fn group_by<C>(self, column: C) -> Self
    where
        C: Column,
    {
        self.group_by_condition(true, column)
    }

    pub fn group_by_condition<C>(self, condition: bool, column: C) -> Self
    where
        C: Column,
    {
        let Ok(c) = column.alias() else { return self };
        self.do_it(condition, SegmentType::GroupBy, Segment::GroupBy(vec![c]))
    }

    pub fn having(self, sql_having: String) -> Self {
        self.having_condition(true, sql_having)
    }

    pub fn having_condition(self, condition: bool, sql_having: String) -> Self {
        self.do_it(
            condition,
            SegmentType::Having,
            Segment::Having(vec![sql_having]),
        )
    }

    pub fn order_by<C>(self, column: C, is_asc: bool) -> Self
    where
        C: Column,
    {
        self.order_by_condition(true, column, is_asc)
    }

    pub fn order_by_asc<C>(self, column: C) -> Self
    where
        C: Column,
    {
        self.order_by_condition(true, column, true)
    }

    pub fn order_by_desc<C>(self, column: C) -> Self
    where
        C: Column,
    {
        self.order_by_condition(true, column, false)
    }

    pub fn order_by_condition<C>(self, condition: bool, column: C, is_asc: bool) -> Self
    where
        C: Column,
    {
        let mode = if is_asc {
            OrderByType::Asc
        } else {
            OrderByType::Desc
        };
        let Ok(c) = column.alias() else { return self };
        self.do_it(
            condition,
            SegmentType::OrderBy,
            Segment::OrderBy(vec![(c, mode)]),
        )
    }
}

#[macro_export]
macro_rules! add_condition {
    ($self:expr, $column:expr, $val:expr, $fun: ident) => {
        $self.do_it(
            true,
            SegmentType::Normal,
            Segment::$fun($column.alias_unwrap(), $val.into()),
        )
    };
}
