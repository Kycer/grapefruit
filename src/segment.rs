use std::collections::HashMap;

use crate::{Value, Wrapper};

#[derive(Clone, Debug, PartialEq)]
pub enum SegmentType {
    GroupBy,
    Having,
    OrderBy,
    Normal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Or,
    In(String, Value),
    NotIn(String, Value),
    // Not,
    Like(String, Value),
    LikeLeft(String, Value),
    LikeRight(String, Value),
    NotLike(String, Value),
    NotLikeLeft(String, Value),
    NotLikeRight(String, Value),
    Eq(String, Value),
    Ne(String, Value),
    Gt(String, Value),
    Ge(String, Value),
    Lt(String, Value),
    Le(String, Value),
    IsNull(String),
    IsNotNull(String),
    GroupBy(Vec<String>),
    Having(Vec<String>),
    Bracket(Box<Wrapper>),
    OrderBy(Vec<(String, OrderByType)>),
    // EXISTS,
    Between(String, Value, Value),
    NotBetween(String, Value, Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderByType {
    Asc,
    Desc,
}

impl OrderByType {
    pub fn to_string(&self) -> String {
        match self {
            OrderByType::Asc => "ASC".to_string(),
            OrderByType::Desc => "DESC".to_string(),
        }
    }
}

impl Segment {
    pub fn add_group_by(&mut self, v: &Vec<String>) {
        match self {
            Segment::GroupBy(cols) => cols.extend_from_slice(v),
            _ => panic!("Segment is not GroupBy"),
        }
    }

    pub fn add_having(&mut self, v: &Vec<String>) {
        match self {
            Segment::Having(cols) => cols.extend_from_slice(v),
            _ => panic!("Segment is not Having"),
        }
    }

    pub fn add_order_by(&mut self, v: &Vec<(String, OrderByType)>) {
        match self {
            Segment::OrderBy(cols) => cols.extend_from_slice(&v),
            _ => panic!("Segment is not OrderBy"),
        }
    }

    pub fn get_group_by(&self) -> Option<Vec<String>> {
        match self {
            Segment::GroupBy(cols) => Some(cols.clone()),
            _ => None,
        }
    }

    pub fn get_having(&self) -> Option<Vec<String>> {
        match self {
            Segment::Having(cols) => Some(cols.clone()),
            _ => None,
        }
    }

    pub fn get_order_by(&self) -> Option<Vec<(String, OrderByType)>> {
        match self {
            Segment::OrderBy(cols) => Some(cols.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segments {
    pub bracket_num: i32,
    pub normal: Vec<Segment>,
    pub group_by: Segment,
    pub order_by: Segment,
    pub having: Segment,
}

impl Segments {
    pub fn with_bracket_num(bracket_num: i32) -> Self {
        Self {
            bracket_num: bracket_num,
            normal: Vec::new(),
            group_by: Segment::GroupBy(vec![]),
            order_by: Segment::OrderBy(vec![]),
            having: Segment::Having(vec![]),
        }
    }
    pub fn add(&mut self, segment_type: SegmentType, segment: Segment) {
        match segment_type {
            SegmentType::GroupBy => {
                if let Some(col) = segment.get_group_by() {
                    self.group_by.add_group_by(&col);
                }
            }
            SegmentType::Having => {
                if let Some(col) = segment.get_having() {
                    self.having.add_having(&col);
                }
            }
            SegmentType::OrderBy => {
                if let Some(col) = segment.get_order_by() {
                    self.order_by.add_order_by(&col);
                }
            }
            SegmentType::Normal => self.normal.push(segment),
        }
    }

    fn format_col_name(&self, index: usize, col: &str) -> String {
        format!("{}_{}_{}", self.bracket_num, index, col)
    }

    fn get_normal_sql(&self) -> (String, HashMap<String, Value>) {
        let mut sql = String::new();
        let mut params = HashMap::new();
        if self.normal.is_empty() {
            return (sql, params);
        }
        let mut op_is_or = false;
        for (index, segment) in self.normal.iter().enumerate() {
            if !op_is_or && index != 0 && !matches!(segment, Segment::Or) {
                sql.push_str(" and ");
                op_is_or = false;
            }
            match segment {
                Segment::Or => {
                    sql.push_str(" or ");
                    op_is_or = true;
                }
                Segment::In(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} in ( :{} )", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::NotIn(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} not in ( :{} )", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Like(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("%{}%", v))));
                }
                Segment::LikeLeft(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("%{}", v))));
                }
                Segment::LikeRight(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("{}%", v))));
                }

                Segment::NotLike(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} not like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("%{}%", v))));
                }
                Segment::NotLikeLeft(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} not like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("%{}", v))));
                }
                Segment::NotLikeRight(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} not like :{}", col, col_name));
                    let v = val.get_str();
                    params.insert(col_name, Value::String(Some(format!("{}%", v))));
                }
                Segment::Eq(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} = :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Ne(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} <> :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Gt(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} > :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Ge(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} >= :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Lt(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} < :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::Le(col, val) => {
                    let col_name = self.format_col_name(index, col);
                    sql.push_str(&format!("{} <= :{}", col, col_name));
                    params.insert(col_name, val.clone());
                }
                Segment::IsNull(col) => {
                    sql.push_str(&format!("{} is null ", col));
                }
                Segment::IsNotNull(col) => {
                    sql.push_str(&format!("{} is not null ", col));
                }
                Segment::Bracket(w) => {
                    let (s, p) = w.get_sql();
                    sql.push_str(&format!("( {} )", s));
                    params.extend(p);
                }
                Segment::Between(col, val1, val2) => {
                    let col_name = self.format_col_name(index, col);
                    let startcol = format!("{}_start", col_name);
                    let endcol = format!("{}_end", col_name);
                    sql.push_str(&format!("{} between :{} and :{} ", col, startcol, endcol));
                    params.insert(startcol, val1.clone());
                    params.insert(endcol, val2.clone());
                }
                Segment::NotBetween(col, val1, val2) => {
                    let col_name = self.format_col_name(index, col);
                    let startcol = format!("{}_start", col_name);
                    let endcol = format!("{}_end", col_name);
                    sql.push_str(&format!(
                        "{} not between :{} and :{} ",
                        col, startcol, endcol
                    ));
                    params.insert(startcol, val1.clone());
                    params.insert(endcol, val2.clone());
                }
                _ => {}
            }
        }
        (sql, params)
    }

    fn get_group_by_sql(&self) -> String {
        if let Some(v) = self.group_by.get_group_by() {
            if v.is_empty() {
                return "".to_string();
            }
            format!("group by {}", v.join(","))
        } else {
            "".to_string()
        }
    }

    fn get_having_sql(&self) -> String {
        if let Some(v) = self.having.get_having() {
            if v.is_empty() {
                return "".to_string();
            }
            format!("having {}", v.join(" and "))
        } else {
            "".to_string()
        }
    }

    fn get_order_by_sql(&self) -> String {
        if let Some(v) = self.order_by.get_order_by() {
            if v.is_empty() {
                return "".to_string();
            }
            let str = v
                .iter()
                .map(|vv| format!("{} {}", vv.clone().0, vv.1.to_string()))
                .collect::<Vec<_>>();
            format!("order by {}", str.join(","))
        } else {
            "".to_string()
        }
    }

    pub fn get_sql(&self) -> (String, HashMap<String, Value>) {
        let (normal_sql, val) = self.get_normal_sql();
        let group_by = self.get_group_by_sql();
        let having = self.get_having_sql();
        let order_by = self.get_order_by_sql();
        let sql = format!("{} {} {} {}", normal_sql, group_by, having, order_by);
        (sql, val)
    }
}
