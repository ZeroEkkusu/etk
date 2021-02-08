use crate::ops::{Op, TryFromIntError};

use std::collections::{hash_map, HashMap, VecDeque};

#[derive(Debug)]
pub enum Error {
    DuplicateLabel,
    LabelTooLarge,
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Error::LabelTooLarge
    }
}

#[derive(Debug, Default)]
pub struct Assembler {
    ready: Vec<u8>,
    pending: VecDeque<Op>,
    code_len: u32,
    labels: HashMap<String, u32>,
}

impl Assembler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.ready)
    }

    pub fn push_all<I, O>(&mut self, ops: I) -> Result<usize, Error>
    where
        I: IntoIterator<Item = O>,
        O: Into<Op>,
    {
        let ops = ops.into_iter().map(Into::into);

        for op in ops {
            self.push(op)?;
        }

        Ok(self.ready.len())
    }

    pub fn push(&mut self, op: Op) -> Result<usize, Error> {
        let specifier = op.specifier();

        if let Some(label) = op.label() {
            match self.labels.entry(label.to_owned()) {
                hash_map::Entry::Occupied(_) => return Err(Error::DuplicateLabel),
                hash_map::Entry::Vacant(v) => {
                    v.insert(self.code_len);
                }
            }
        }

        if self.pending.is_empty() {
            self.push_ready(op)?;
        } else {
            self.push_pending(op)?;
        }

        self.code_len += 1 + specifier.extra_len();
        Ok(self.ready.len())
    }

    fn push_ready(&mut self, mut op: Op) -> Result<(), Error> {
        if let Some(label) = op.immediate_label() {
            match self.labels.get(label) {
                Some(addr) => op = op.realize(*addr)?,
                None => {
                    self.pending.push_back(op);
                    return Ok(());
                }
            }
        }

        op.assemble(&mut self.ready);

        Ok(())
    }

    fn push_pending(&mut self, op: Op) -> Result<(), Error> {
        self.pending.push_back(op);

        while let Some(next) = self.pending.front() {
            let mut address = None;

            if let Some(label) = next.immediate_label() {
                match self.labels.get(label) {
                    Some(addr) => address = Some(*addr),
                    None => break,
                }
            }

            let popped = match address {
                Some(s) => {
                    // Don't modify `self.pending` if realize returns an error.
                    let realized = next.realize(s)?;
                    self.pending.pop_front();
                    realized
                }
                None => self.pending.pop_front().unwrap(),
            };

            popped.assemble(&mut self.ready);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn assemble_jumpdest_no_label() -> Result<(), Error> {
        let mut asm = Assembler::new();
        let sz = asm.push_all(vec![Op::JumpDest(None)])?;
        assert_eq!(1, sz);
        assert!(asm.labels.is_empty());
        assert_eq!(asm.take(), hex!("5b"));
        Ok(())
    }

    #[test]
    fn assemble_jumpdest_with_label() -> Result<(), Error> {
        let mut asm = Assembler::new();
        let op = Op::JumpDest(Some("lbl".into()));

        let sz = asm.push_all(vec![op])?;
        assert_eq!(1, sz);
        assert_eq!(asm.labels.len(), 1);
        assert_eq!(asm.labels.get("lbl"), Some(&0));
        assert_eq!(asm.take(), hex!("5b"));
        Ok(())
    }

    #[test]
    fn assemble_jumpdest_jump_with_label() -> Result<(), Error> {
        let ops = vec![Op::JumpDest(Some("lbl".into())), Op::Push1("lbl".into())];

        let mut asm = Assembler::new();
        let sz = asm.push_all(ops)?;
        assert_eq!(sz, 3);
        assert_eq!(asm.take(), hex!("5b6000"));

        Ok(())
    }

    #[test]
    fn assemble_jump_jumpdest_with_label() -> Result<(), Error> {
        let ops = vec![Op::Push1("lbl".into()), Op::JumpDest(Some("lbl".into()))];

        let mut asm = Assembler::new();
        let sz = asm.push_all(ops)?;
        assert_eq!(sz, 3);
        assert_eq!(asm.take(), hex!("60025b"));

        Ok(())
    }
}
