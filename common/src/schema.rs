pub trait Schema {
    const TABLE: &'static str;
    const SELECT: &'static str = "*";
}

