//! OGC GeoPackage (1.4) writer for ArcGIS Pro / QGIS: one feature table per GeoJSON `kind`.

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params_from_iter, types::Value as Sql};
use serde_json::{Map, Value};
use std::path::Path;

/// GeoJSON `kind` → layer name, in display order.
const LAYERS: [(&str, &str); 4] = [("cell", "cells"), ("candidate", "candidates"), ("ellipse", "ellipses"), ("onset", "onsets")];
const WGS84: i32 = 4326;
const WGS84_WKT: &str = r#"GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433],AUTHORITY["EPSG","4326"]]"#;

/// Writes the features of `fc` (as built by `geojson::feature_collection`) to a fresh GeoPackage at `path`.
pub fn write(path: &Path, fc: &Value) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path).with_context(|| format!("replacing {}", path.display()))?;
    }
    let mut db = Connection::open(path)?;
    db.execute_batch(SCHEMA)?;
    db.execute(
        "INSERT INTO gpkg_spatial_ref_sys VALUES ('WGS 84', ?1, 'EPSG', ?1, ?2, 'WGS 84 geographic')",
        (WGS84, WGS84_WKT),
    )?;
    let tx = db.transaction()?;
    let features = fc["features"].as_array().context("not a FeatureCollection")?;
    for (kind, layer) in LAYERS {
        let rows: Vec<&Value> = features.iter().filter(|f| f["properties"]["kind"] == kind).collect();
        if !rows.is_empty() {
            write_layer(&tx, layer, &rows)?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn write_layer(db: &Connection, layer: &str, features: &[&Value]) -> Result<()> {
    let flat: Vec<Vec<(String, Value)>> = features.iter().map(|f| flatten(&f["properties"])).collect();
    let mut cols: Vec<(String, &str)> = Vec::new();
    for row in &flat {
        for (k, v) in row {
            match cols.iter_mut().find(|(c, _)| c == k) {
                Some(col) if col.1 == "NULL" => col.1 = sql_type(v),
                Some(_) => {}
                None => cols.push((k.clone(), sql_type(v))),
            }
        }
    }
    let ty = |t: &'static str| if t == "NULL" { "TEXT" } else { t };
    let col_defs: String = cols.iter().map(|(c, t)| format!(", \"{c}\" {}", ty(t))).collect();
    let geom_type = features[0]["geometry"]["type"].as_str().unwrap_or("GEOMETRY").to_uppercase();
    db.execute_batch(&format!("CREATE TABLE \"{layer}\" (fid INTEGER PRIMARY KEY AUTOINCREMENT, geom {geom_type}{col_defs});"))?;

    let names: String = cols.iter().map(|(c, _)| format!(", \"{c}\"")).collect();
    let marks: String = cols.iter().map(|_| ", ?").collect();
    let mut ins = db.prepare(&format!("INSERT INTO \"{layer}\" (geom{names}) VALUES (?{marks})"))?;
    let mut bbox = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for (f, row) in features.iter().zip(&flat) {
        let mut vals = vec![Sql::Blob(geometry_blob(&f["geometry"], &mut bbox)?)];
        vals.extend(cols.iter().map(|(c, _)| row.iter().find(|(k, _)| k == c).map_or(Sql::Null, |(_, v)| to_sql(v))));
        ins.execute(params_from_iter(vals))?;
    }
    db.execute(
        "INSERT INTO gpkg_contents (table_name, data_type, identifier, min_x, min_y, max_x, max_y, srs_id) VALUES (?1, 'features', ?1, ?2, ?3, ?4, ?5, ?6)",
        (layer, bbox[0], bbox[1], bbox[2], bbox[3], WGS84),
    )?;
    db.execute("INSERT INTO gpkg_geometry_columns VALUES (?1, 'geom', ?2, ?3, 0, 0)", (layer, &geom_type, WGS84))?;
    Ok(())
}

/// Scalar properties as columns; nested objects become `parent_child` columns. `kind` is the layer itself.
fn flatten(props: &Value) -> Vec<(String, Value)> {
    fn walk(prefix: &str, obj: &Map<String, Value>, out: &mut Vec<(String, Value)>) {
        for (k, v) in obj {
            let name = if prefix.is_empty() { k.clone() } else { format!("{prefix}_{k}") };
            match v {
                Value::Object(o) => walk(&name, o, out),
                Value::Array(_) => out.push((name, Value::String(v.to_string()))),
                _ if name == "kind" => {}
                _ => out.push((name, v.clone())),
            }
        }
    }
    let mut out = Vec::new();
    if let Some(o) = props.as_object() {
        walk("", o, &mut out);
    }
    out
}

fn sql_type(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "BOOLEAN",
        Value::Number(n) if n.is_i64() || n.is_u64() => "INTEGER",
        Value::Number(_) => "DOUBLE",
        Value::String(_) => "TEXT",
        _ => "NULL",
    }
}

fn to_sql(v: &Value) -> Sql {
    match v {
        Value::Bool(b) => Sql::Integer(*b as i64),
        Value::Number(n) => n.as_i64().map_or_else(|| Sql::Real(n.as_f64().unwrap_or(f64::NAN)), Sql::Integer),
        Value::String(s) => Sql::Text(s.clone()),
        _ => Sql::Null,
    }
}

/// GeoPackage binary: "GP" header (v0, little-endian, no envelope, srs id) followed by little-endian WKB.
fn geometry_blob(geom: &Value, bbox: &mut [f64; 4]) -> Result<Vec<u8>> {
    let mut b = vec![b'G', b'P', 0, 0b0000_0001];
    b.extend(WGS84.to_le_bytes());
    b.push(1);
    let mut point = |b: &mut Vec<u8>, c: &Value| -> Result<()> {
        let (x, y) = (c[0].as_f64().context("bad coordinate")?, c[1].as_f64().context("bad coordinate")?);
        *bbox = [bbox[0].min(x), bbox[1].min(y), bbox[2].max(x), bbox[3].max(y)];
        b.extend(x.to_le_bytes());
        b.extend(y.to_le_bytes());
        Ok(())
    };
    let coords = &geom["coordinates"];
    match geom["type"].as_str() {
        Some("Point") => {
            b.extend(1u32.to_le_bytes());
            point(&mut b, coords)?;
        }
        Some("Polygon") => {
            let rings = coords.as_array().context("bad polygon")?;
            b.extend(3u32.to_le_bytes());
            b.extend((rings.len() as u32).to_le_bytes());
            for ring in rings {
                let pts = ring.as_array().context("bad ring")?;
                b.extend((pts.len() as u32).to_le_bytes());
                for p in pts {
                    point(&mut b, p)?;
                }
            }
        }
        other => bail!("unsupported geometry {other:?}"),
    }
    Ok(b)
}

const SCHEMA: &str = r#"
PRAGMA application_id = 1196444487;
PRAGMA user_version = 10400;
CREATE TABLE gpkg_spatial_ref_sys (
  srs_name TEXT NOT NULL, srs_id INTEGER PRIMARY KEY, organization TEXT NOT NULL,
  organization_coordsys_id INTEGER NOT NULL, definition TEXT NOT NULL, description TEXT);
INSERT INTO gpkg_spatial_ref_sys VALUES
  ('Undefined cartesian SRS', -1, 'NONE', -1, 'undefined', 'undefined cartesian coordinate reference system'),
  ('Undefined geographic SRS', 0, 'NONE', 0, 'undefined', 'undefined geographic coordinate reference system');
CREATE TABLE gpkg_contents (
  table_name TEXT NOT NULL PRIMARY KEY, data_type TEXT NOT NULL, identifier TEXT UNIQUE, description TEXT DEFAULT '',
  last_change DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  min_x DOUBLE, min_y DOUBLE, max_x DOUBLE, max_y DOUBLE,
  srs_id INTEGER, CONSTRAINT fk_gc_r_srs_id FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys(srs_id));
CREATE TABLE gpkg_geometry_columns (
  table_name TEXT NOT NULL, column_name TEXT NOT NULL, geometry_type_name TEXT NOT NULL,
  srs_id INTEGER NOT NULL, z TINYINT NOT NULL, m TINYINT NOT NULL,
  CONSTRAINT pk_geom_cols PRIMARY KEY (table_name, column_name),
  CONSTRAINT fk_gc_tn FOREIGN KEY (table_name) REFERENCES gpkg_contents(table_name),
  CONSTRAINT fk_gc_srs FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys(srs_id));
"#;
