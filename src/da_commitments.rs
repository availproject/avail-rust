use kate::{
    couscous::multiproof_params,
    gridgen::{AsBytes, EvaluationGrid},
    pmp::m1_blst::M1NoPrecomp,
    Seed,
};
use std::sync::OnceLock;
use thiserror_no_std::Error;

pub type DaCommitments = Vec<u8>;

static PUBLIC_PARAMS: OnceLock<M1NoPrecomp> = OnceLock::new();

#[derive(Error, Debug)]
pub enum DaCommitmentsError {
    #[error("Grid construction failed: {0}")]
    GridConstructionFailed(String),
    #[error("Make polynomial grid failed: {0}")]
    MakePolynomialGridFailed(String),
    #[error("Grid extension failed: {0}")]
    GridExtensionFailed(String),
    #[error("Commitment serialization failed: {0}")]
    CommitmentSerializationFailed(String),
}

pub struct DaCommitmentBuilder {
    data: Vec<u8>,
    max_width: usize,
    max_height: usize,
    seed: Seed,
    public_params: &'static M1NoPrecomp,
}

impl DaCommitmentBuilder {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            max_width: 1024,
            max_height: 1024,
            seed: Seed::default(),
            public_params: PUBLIC_PARAMS.get_or_init(multiproof_params),
        }
    }

    pub fn max_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: usize) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    pub fn build(self) -> Result<DaCommitments, DaCommitmentsError> {
        let grid = self.build_grid()?;
        self.build_commitment(&grid)
    }

    fn build_grid(&self) -> Result<EvaluationGrid, DaCommitmentsError> {
        EvaluationGrid::from_data(
            self.data.clone(),
            self.max_width,
            self.max_width,
            self.max_height,
            self.seed,
        )
        .map_err(|e| DaCommitmentsError::GridConstructionFailed(format!("{:?}", e)))
    }

    fn build_commitment(&self, grid: &EvaluationGrid) -> Result<Vec<u8>, DaCommitmentsError> {
        let poly_grid = grid
            .make_polynomial_grid()
            .map_err(|e| DaCommitmentsError::MakePolynomialGridFailed(format!("{:?}", e)))?;

        let extended_grid = poly_grid
            .commitments(self.public_params) // Direct reference, no Arc needed
            .map_err(|e| DaCommitmentsError::GridExtensionFailed(format!("{:?}", e)))?;

        let commitment = extended_grid
            .iter()
            .filter_map(|c| c.to_bytes().ok())
            .flatten()
            .collect::<Vec<u8>>();

        if commitment.is_empty() {
            return Err(DaCommitmentsError::CommitmentSerializationFailed(
                "Failed to serialize commitments".to_string(),
            ));
        }

        Ok(commitment)
    }
}
