mod number;

pub trait PerEncodable {
    pub fn encode(&self) -> BitVec;
}
