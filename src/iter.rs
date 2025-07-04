#[derive(Debug, Copy, Clone)]
pub enum SingleError {
    None,
    Multiple,
}

pub trait IterExt {
    type Item;
    
    fn expect_single(&mut self) -> Result<Self::Item, SingleError>;
}

impl <T: Iterator> IterExt for T {
    type Item = T::Item;

    fn expect_single(&mut self) -> Result<Self::Item, SingleError> {
        let first = self.next();
        
        if first.is_none() {
            return Err(SingleError::None);
        }
        
        let second = self.next();
        
        if second.is_some() {
            return Err(SingleError::Multiple);
        }
        
        Ok(first.unwrap())
    }
}