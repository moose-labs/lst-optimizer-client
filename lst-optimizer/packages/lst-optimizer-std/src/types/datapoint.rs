pub trait DatapointFetcher<T> {
    fn fetch(&self) -> Vec<SymbolData<T>>;
}

pub trait Datapoint {
    fn get_symbol(&self) -> String;
}

pub struct SymbolData<T> {
    pub mint: String,
    pub symbol: String,
    pub datapoints: Vec<T>,
}
