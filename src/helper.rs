use crate::{Column, Entity, Grapefruit, GrapefruitResult, Params, Platform, Value};

pub static COMMA: &str = ",";

#[inline]
pub async fn build_insert_sql<T>(
    entities: &[&T],
    grapefruit: &Grapefruit,
) -> GrapefruitResult<(String, Params)>
where
    T: Entity,
{
    let column_map = T::columns();
    let insert_columns = T::insert_columns();
    let mut values = Vec::new();
    let mut marks_str = Vec::new();
    let mut i = 1;
    for entity in entities.into_iter() {
        let data = entity.to_value();
        let mut marks = Vec::new();
        for column in insert_columns.iter() {
            let column_info = column_map.get(column).unwrap();
            let value = data
                .get(column)
                .expect(format!("{} column value not found.", column.as_str()).as_str());
            if column_info.is_table_id() {
                match column_info.id_type() {
                    crate::IdType::Auto => {}
                    crate::IdType::Generator => {
                        let id = grapefruit.generator_id().await;
                        values.push(id);
                    }
                    crate::IdType::Input => {
                        values.push(value.clone());
                    }
                }
            } else {
                match column_info.fill {
                    crate::Fill::Insert | crate::Fill::InsertAndUpdate => {
                        let v = &grapefruit.get_insert_fill(&column)?;
                        values.push(v.clone());
                    }
                    _ => {
                        values.push(value.clone());
                    }
                }
            }
            marks.push(grapefruit.platform().mark(i));
            i += 1;
        }
        marks_str.push(format!("({})", marks.join(",")));
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES {} ",
        T::table_name(),
        insert_columns.join(","),
        marks_str.join(",")
    );
    Ok((sql, values.into()))
}

#[inline]
pub async fn build_update_sql<T, F>(
    entity: &T,
    grapefruit: &Grapefruit,
    f: F,
) -> GrapefruitResult<(String, Params)>
where
    T: Entity,
    F: Fn(usize) -> (String, Vec<Value>),
{
    let column_map = T::columns();
    let data = entity.to_value();
    let update_columns = T::update_columns();
    let mut values = Vec::with_capacity(update_columns.len() + 1);
    let mut columns = Vec::with_capacity(update_columns.len());
    for (index, column) in update_columns.iter().enumerate() {
        let value = data.get(column).unwrap();
        let column_info = column_map.get(column).unwrap();
        match column_info.fill {
            crate::Fill::Insert | crate::Fill::InsertAndUpdate => {
                let v = &grapefruit.get_insert_fill(&column)?;
                values.push(v.clone());
            }
            _ => {
                values.push(value.clone());
            }
        }
        columns.push(format!(
            "{} = {}",
            grapefruit.platform().symbol(column),
            grapefruit.platform().mark(index + 1)
        ));
    }

    let (build_sql, vals) = f(columns.len());
    values.extend_from_slice(&vals);
    let sql = format!(
        "UPDATE {} SET {} WHERE {} ",
        T::table_name(),
        columns.join(","),
        build_sql,
    );
    println!("{:?}", sql);
    println!("{:?}", values);
    Ok(build_logic_delete::<T>(sql, values, grapefruit.platform()))
}

#[inline]
pub async fn build_delete_sql<T, F>(
    grapefruit: &Grapefruit,
    f: F,
) -> GrapefruitResult<(String, Params)>
where
    T: Entity,
    F: Fn(usize) -> (String, Vec<Value>),
{
    let logic_delete = T::logic_delete();

    let (sql, vals) = match logic_delete {
        Some(v) => {
            let (build_sql, mut vals) = f(1);
            let (_, value) = v.column_type_unwrap().logic_delete_value();
            let sql = format!(
                "UPDATE {} SET {} = {} WHERE {} ",
                T::table_name(),
                v.alias_unwrap(),
                grapefruit.platform().mark(1),
                build_sql,
            );
            vals.insert(0, value);
            (sql, vals)
        }
        None => {
            let (build_sql, vals) = f(0);
            let sql = format!("DELETE FROM {} WHERE {}", T::table_name(), build_sql,);
            (sql, vals)
        }
    };

    Ok(build_logic_delete::<T>(sql, vals, grapefruit.platform()))
}

#[inline]
pub async fn build_select_sql<T, F>(
    grapefruit: &Grapefruit,
    f: F,
) -> GrapefruitResult<(String, Params)>
where
    T: Entity,
    F: Fn(usize) -> (String, Vec<Value>),
{
    let (build_sql, vals) = f(0);
    let select_colums = T::select_columns();
    let sql = format!(
        "SELECT {} FROM {}  WHERE {}",
        select_colums.join(","),
        T::table_name(),
        build_sql,
    );

    Ok(build_logic_delete::<T>(sql, vals, grapefruit.platform()))
}

#[inline]
pub fn build_logic_delete<T>(
    sql: String,
    mut vals: Vec<Value>,
    platform: &Platform,
) -> (String, Params)
where
    T: Entity,
{
    match T::logic_delete() {
        Some(logic) => {
            let (value, _) = logic.column_type_unwrap().logic_delete_value();
            let sql = format!(
                "{} AND {} = {}",
                sql,
                logic.alias_unwrap(),
                platform.mark(vals.len() + 1)
            );
            vals.push(value);
            (sql, vals.into())
        }
        None => (sql, vals.into()),
    }
}
