pub mod constants;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenlNbdCmd {

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenlNbd {
    /// Command code of this message
    pub cmd: GenlNbdCmd,
    /// Netlink attributes in this message
    pub nlas: Vec<GenlNbdAttrs>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenlNbdAttrs {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
