use crate::U256;
use codec::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use sp_core::ConstU32;

/// Compatible with `kate::com::Cell`
#[derive(Clone, Constructor, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Cell {
	#[codec(compact)]
	pub row: u32,
	#[codec(compact)]
	pub col: u32,
}

impl<R, C> From<(R, C)> for Cell
where
	R: Into<u32>,
	C: Into<u32>,
{
	fn from((row, col): (R, C)) -> Self {
		Self {
			row: row.into(),
			col: col.into(),
		}
	}
}

pub type GRawScalar = U256;
pub type GRow = Vec<GRawScalar>;
pub type GDataProof = (GRawScalar, GProof);
pub type GMultiProof = (Vec<GRawScalar>, GProof);
pub type MaxCells = ConstU32<64>;
pub type Cells = bounded_collections::BoundedVec<Cell, MaxCells>;
pub type MaxRows = ConstU32<64>;
pub type Rows = bounded_collections::BoundedVec<u32, MaxRows>;

#[derive(Encode, Decode, Debug, Clone, Serialize, Deserialize)]
pub struct GCellBlock {
	pub start_x: u32,
	pub start_y: u32,
	pub end_x: u32,
	pub end_y: u32,
}

impl GCellBlock {
    pub const GCELL_BLOCK_SIZE: usize = std::mem::size_of::<GCellBlock>();

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16);
        buf.extend(&self.start_x.to_le_bytes());
        buf.extend(&self.start_y.to_le_bytes());
        buf.extend(&self.end_x.to_le_bytes());
        buf.extend(&self.end_y.to_le_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != Self::GCELL_BLOCK_SIZE {
            return Err("GCellBlock must be exactly 16 bytes");
        }

        let start_x = bytes
            .get(0..4)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes);
        let start_y = bytes
            .get(4..8)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes);
        let end_x = bytes
            .get(8..12)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes);
        let end_y = bytes
            .get(12..16)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes);

        match (start_x, start_y, end_x, end_y) {
            (Some(start_x), Some(start_y), Some(end_x), Some(end_y)) => Ok(Self {
                start_x,
                start_y,
                end_x,
                end_y,
            }),
            _ => Err("Failed to convert bytes to GCellBlock"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "Vec<u8>", into = "Vec<u8>")]
pub struct GProof(pub [u8; 48]);

impl From<GProof> for Vec<u8> {
	fn from(proof: GProof) -> Self {
		proof.0.to_vec()
	}
}

impl TryFrom<Vec<u8>> for GProof {
	type Error = u32;
	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		if data.len() != 48 {
			return Err(data.len() as u32);
		};

		let mut proof = [0u8; 48];
		proof.copy_from_slice(&data);
		Ok(GProof(proof))
	}
}
