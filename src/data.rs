extern crate anyhow;
extern crate ipfs_embed;
extern crate libipld;

use crate::rpc::get_kate_query_proof_by_cell;
use ipfs_embed::{Block, DefaultParams};
use ipfs_embed::{Cid, DefaultParams as IPFSDefaultParams, Ipfs, TempPin};
use libipld::codec_impl::IpldCodec;
use libipld::multihash::Code;
use libipld::Ipld;

pub type IpldBlock = Block<DefaultParams>;
pub type BaseCell = IpldBlock;

#[derive(Clone)]
pub struct L0Col {
    pub base_cells: Vec<BaseCell>,
}

#[derive(Clone)]
pub struct L1Row {
    pub l0_cols: Vec<L0Col>,
}

#[derive(Clone)]
pub struct DataMatrix {
    pub block_num: i128,
    pub l1_row: L1Row,
}

pub async fn construct_cell(block: u64, row: u16, col: u16) -> BaseCell {
    let data = Ipld::Bytes(get_kate_query_proof_by_cell(block, row, col).await);
    IpldBlock::encode(IpldCodec::DagCbor, Code::Blake3_256, &data).unwrap()
}

pub async fn construct_colwise(block: u64, row_count: u16, col: u16) -> L0Col {
    let mut base_cells: Vec<BaseCell> = Vec::with_capacity(row_count as usize);

    for row in 0..row_count {
        base_cells.push(construct_cell(block, row, col).await);
    }

    L0Col {
        base_cells: base_cells,
    }
}

pub async fn construct_rowwise(block: u64, row_count: u16, col_count: u16) -> L1Row {
    let mut l0_cols: Vec<L0Col> = Vec::with_capacity(col_count as usize);

    for col in 0..col_count {
        l0_cols.push(construct_colwise(block, row_count, col).await);
    }

    L1Row { l0_cols: l0_cols }
}

pub async fn construct_matrix(block: u64, row_count: u16, col_count: u16) -> DataMatrix {
    DataMatrix {
        l1_row: construct_rowwise(block, row_count, col_count).await,
        block_num: block as i128,
    }
}

pub async fn push_cell(
    cell: BaseCell,
    ipfs: &Ipfs<IPFSDefaultParams>,
    pin: &TempPin,
) -> anyhow::Result<Cid> {
    ipfs.temp_pin(pin, cell.cid())?;
    ipfs.insert(&cell)?;

    Ok(*cell.cid())
}

pub async fn push_col(
    col: L0Col,
    ipfs: &Ipfs<DefaultParams>,
    pin: &TempPin,
) -> anyhow::Result<Cid> {
    let mut cell_cids: Vec<Ipld> = Vec::with_capacity(col.base_cells.len());

    for cell in col.base_cells {
        if let Ok(cid) = push_cell(cell, ipfs, pin).await {
            cell_cids.push(Ipld::Link(cid));
        };
    }

    let col = Ipld::List(cell_cids);
    let coded_col = IpldBlock::encode(IpldCodec::DagCbor, Code::Blake3_256, &col).unwrap();

    ipfs.temp_pin(pin, coded_col.cid())?;
    ipfs.insert(&coded_col)?;

    Ok(*coded_col.cid())
}
