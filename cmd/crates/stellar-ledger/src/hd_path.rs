#[derive(Clone, Copy)]
pub struct HdPath(pub u32);

impl HdPath {
    #[must_use]
    pub fn depth(&self) -> u8 {
        let path: slip10::BIP32Path = self.into();
        path.depth()
    }
}

impl From<u32> for HdPath {
    fn from(index: u32) -> Self {
        HdPath(index)
    }
}

impl From<&u32> for HdPath {
    fn from(index: &u32) -> Self {
        HdPath(*index)
    }
}

impl HdPath {
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        hd_path_to_bytes(&self.into())
    }
}

impl From<&HdPath> for slip10::BIP32Path {
    fn from(value: &HdPath) -> Self {
        let index = value.0;
        format!("m/44'/148'/{index}'").parse().unwrap()
    }
}

fn hd_path_to_bytes(hd_path: &slip10::BIP32Path) -> Vec<u8> {
    let hd_path_indices = 0..hd_path.depth();
    // Unsafe unwrap is safe because the depth is the length of interneal vector
    hd_path_indices
        .into_iter()
        .flat_map(|index| unsafe { hd_path.index(index).unwrap_unchecked().to_be_bytes() })
        .collect()
}
