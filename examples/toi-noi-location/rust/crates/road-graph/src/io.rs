//! Định dạng nhị phân có phiên bản cho đồ thị đã biên dịch (`data/hcm-graph.bin`).
//!
//! Bố cục: `"TNGRAPH\0"` (8 byte) · phiên bản `u32` · độ dài phần meta `u64` · meta (postcard: tên, cấm rẽ, cổng, số đo)
//! · các cột mảng, mỗi cột là số phần tử `u64` rồi byte thô little-endian. Nạp chỉ là đọc file và sao chép từng cột,
//! không phân tích gì trên đường nóng.

use crate::model::{Entrance, Graph, Meta, Restriction};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

#[cfg(not(target_endian = "little"))]
compile_error!("định dạng file đồ thị giả định little-endian");

pub const MAGIC: &[u8; 8] = b"TNGRAPH\0";
pub const VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum GraphIoError {
    #[error("lỗi I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("không phải file đồ thị Tới Nơi (sai magic)")]
    BadMagic,
    #[error(
        "phiên bản file {found}, mã nguồn hiểu phiên bản {expected}; biên dịch lại bằng lab LOC-06"
    )]
    Version { found: u32, expected: u32 },
    #[error("file bị cắt cụt")]
    Truncated,
    #[error("phần meta hỏng: {0}")]
    Meta(String),
    #[error("đồ thị không hợp lệ: {0}")]
    Invalid(String),
}

#[derive(Serialize, Deserialize)]
struct Extras {
    meta: Meta,
    names: Vec<String>,
    restrictions: Vec<Restriction>,
    entrances: Vec<Entrance>,
}

macro_rules! columns {
    ($mac:ident) => {
        $mac!(
            node_ll,
            node_osm,
            first_out,
            tail,
            head,
            length_cm,
            modes,
            class,
            flags,
            maxspeed_kmh,
            lanes,
            width_dm,
            layer,
            twin,
            way_id,
            name,
            shape_start,
            shape_len,
            shape
        )
    };
}

/// Ghi đồ thị ra file. Trả số byte đã ghi.
pub fn save(g: &Graph, path: &Path) -> Result<u64, GraphIoError> {
    let extras = Extras {
        meta: g.meta.clone(),
        names: g.names.clone(),
        restrictions: g.restrictions.clone(),
        entrances: g.entrances.clone(),
    };
    let extras_bytes =
        postcard::to_stdvec(&extras).map_err(|e| GraphIoError::Meta(e.to_string()))?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut written = 0u64;
    let mut put = |bytes: &[u8]| -> std::io::Result<()> {
        written += bytes.len() as u64;
        out.write_all(bytes)
    };
    put(MAGIC)?;
    put(&VERSION.to_le_bytes())?;
    put(&(extras_bytes.len() as u64).to_le_bytes())?;
    put(&extras_bytes)?;
    macro_rules! write_cols {
        ($($f:ident),*) => {$(
            put(&(g.$f.len() as u64).to_le_bytes())?;
            put(bytemuck::cast_slice(&g.$f))?;
        )*};
    }
    columns!(write_cols);
    out.flush()?;
    Ok(written)
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], GraphIoError> {
        let end = self.pos.checked_add(n).ok_or(GraphIoError::Truncated)?;
        let s = self.buf.get(self.pos..end).ok_or(GraphIoError::Truncated)?;
        self.pos = end;
        Ok(s)
    }
    fn u64(&mut self) -> Result<u64, GraphIoError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 byte"),
        ))
    }
    fn col<T: bytemuck::Pod>(&mut self) -> Result<Vec<T>, GraphIoError> {
        let n = self.u64()? as usize;
        let bytes = self.take(
            n.checked_mul(size_of::<T>())
                .ok_or(GraphIoError::Truncated)?,
        )?;
        Ok(bytemuck::pod_collect_to_vec(bytes))
    }
}

/// Nạp đồ thị từ file và kiểm tra bất biến CSR.
pub fn load(path: &Path) -> Result<Graph, GraphIoError> {
    let buf = std::fs::read(path)?;
    let mut r = Reader { buf: &buf, pos: 0 };
    if r.take(8)? != MAGIC {
        return Err(GraphIoError::BadMagic);
    }
    let version = u32::from_le_bytes(r.take(4)?.try_into().expect("4 byte"));
    if version != VERSION {
        return Err(GraphIoError::Version {
            found: version,
            expected: VERSION,
        });
    }
    let extras_len = r.u64()? as usize;
    let extras: Extras =
        postcard::from_bytes(r.take(extras_len)?).map_err(|e| GraphIoError::Meta(e.to_string()))?;
    let mut g = Graph {
        names: extras.names,
        restrictions: extras.restrictions,
        entrances: extras.entrances,
        meta: extras.meta,
        ..Graph::default()
    };
    macro_rules! read_cols {
        ($($f:ident),*) => {$( g.$f = r.col()?; )*};
    }
    columns!(read_cols);
    g.validate().map_err(GraphIoError::Invalid)?;
    Ok(g)
}
