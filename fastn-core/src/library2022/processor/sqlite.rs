pub(crate) fn get_p1_data(
    value: &ftd::ast::VariableValue,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<(ftd::ast::HeaderValues, String)> {
    match value.get_record(doc.name) {
        Ok(val) => {
            Ok((
                val.2.to_owned(),
                match val.3 {
                    Some(b) => b.value.clone(),
                    None => return ftd::interpreter::utils::e2(
                        "$processor$: `package-query` query is not specified in the processor body"
                            .to_string(),
                        doc.name,
                        value.line_number(),
                    ),
                },
            ))
        }
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn get_params(
    headers: &ftd::ast::HeaderValues,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<Vec<String>> {
    headers
        .0
        .iter()
        .filter(|hv| hv.key.eq("param"))
        .map(|x| x.value.string(doc.name))
        .collect::<ftd::ast::Result<Vec<String>>>()
        .map_err(|e| e.into())
}

pub async fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'a>,
    config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = get_p1_data(&value, doc)?;

    let sqlite_database =
        match headers.get_optional_string_by_key("db", doc.name, value.line_number())? {
            Some(k) => k,
            None => {
                return ftd::interpreter::utils::e2(
                    "`db` is not specified".to_string(),
                    doc.name,
                    value.line_number(),
                )
            }
        };
    let mut sqlite_database_path = camino::Utf8PathBuf::new().join(sqlite_database.as_str());
    if !sqlite_database_path.exists() {
        if !config.root.join(sqlite_database_path.as_path()).exists() {
            return ftd::interpreter::utils::e2(
                "`db` does not exists for package-query processor".to_string(),
                doc.name,
                value.line_number(),
            );
        }
        sqlite_database_path = config.root.join(sqlite_database_path.as_path());
    }

    // need the query params
    // question is they can be multiple
    // so lets say start with passing attributes from ftd file
    // db-<param-name1>: value
    // db-<param-name2>: value
    // for now they wil be ordered
    // select * from users where

    let query_params = get_params(&headers, doc)?;

    let result = execute_query(
        &sqlite_database_path,
        query.as_str(),
        doc.name,
        value.line_number(),
        query_params,
    )
    .await?;

    if kind.is_list() {
        doc.from_json_rows(result.as_slice(), &kind, value.line_number())
    } else {
        match result.len() {
            1 => doc.from_json_row(&result[0], &kind, value.line_number()),
            0 => ftd::interpreter::utils::e2(
                "Query returned no result, expected one row".to_string(),
                doc.name,
                value.line_number(),
            ),
            len => ftd::interpreter::utils::e2(
                format!("Query returned {} rows, expected one row", len),
                doc.name,
                value.line_number(),
            ),
        }
    }
}

async fn execute_query(
    database_path: &camino::Utf8Path,
    query: &str,
    doc_name: &str,
    line_number: usize,
    query_params: Vec<String>,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let conn = match rusqlite::Connection::open_with_flags(
        database_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(conn) => conn,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to open `{}`: {:?}", database_path, e),
                doc_name,
                line_number,
            );
        }
    };

    let mut stmt = match conn.prepare(query) {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to prepare query: {:?}", e),
                doc_name,
                line_number,
            )
        }
    };

    let count = stmt.column_count();

    // let mut stmt = conn.prepare("SELECT * FROM test where name = :name")?;
    // let mut rows = stmt.query(rusqlite::named_params! { ":name": "one" })?

    // let mut stmt = conn.prepare("SELECT * FROM test where name = ?")?;
    // let mut rows = stmt.query([name])?;

    let mut rows = match stmt.query(rusqlite::params_from_iter(query_params)) {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("Failed to prepare query: {:?}", e),
                doc_name,
                line_number,
            )
        }
    };

    let mut result: Vec<Vec<serde_json::Value>> = vec![];
    loop {
        match rows.next() {
            Ok(None) => break,
            Ok(Some(r)) => {
                result.push(row_to_json(r, count, doc_name, line_number)?);
            }
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Failed to execute query: {:?}", e),
                    doc_name,
                    line_number,
                )
            }
        }
    }
    Ok(result)
}

fn row_to_json(
    r: &rusqlite::Row,
    count: usize,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<serde_json::Value>> {
    let mut row: Vec<serde_json::Value> = vec![];
    for i in 0..count {
        match r.get::<usize, rusqlite::types::Value>(i) {
            Ok(rusqlite::types::Value::Null) => row.push(serde_json::Value::Null),
            Ok(rusqlite::types::Value::Integer(i)) => row.push(serde_json::Value::Number(i.into())),
            Ok(rusqlite::types::Value::Real(i)) => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(i).unwrap(),
            )),
            Ok(rusqlite::types::Value::Text(i)) => row.push(serde_json::Value::String(i)),
            Ok(rusqlite::types::Value::Blob(_)) => {
                return ftd::interpreter::utils::e2(
                    format!("Query returned blob for column: {}", i),
                    doc_name,
                    line_number,
                );
            }
            Err(e) => {
                return ftd::interpreter::utils::e2(
                    format!("Failed to read response: {:?}", e),
                    doc_name,
                    line_number,
                );
            }
        }
    }
    Ok(row)
}
