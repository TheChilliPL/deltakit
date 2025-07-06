use std::mem::MaybeUninit;

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

pub trait ResultArrayExt<const N: usize> {
    type T;
    type E;

    fn flatten_ok(self) -> Result<[Self::T; N], Self::E>; 
}

impl <T, E, const N: usize> ResultArrayExt<N> for [Result<T, E>; N] {
    type T = T;
    type E = E;
    
    fn flatten_ok(self) -> Result<[Self::T; N], Self::E> {
        let mut ok_array = [const { MaybeUninit::uninit() }; N];

        for (i, result) in self.into_iter().enumerate() {
            match result {
                Ok(value) => {
                    unsafe {
                        ok_array[i].write(value);
                    }
                },
                Err(error) => return Err(error),
            }
        }

        unsafe {
            Ok(std::mem::transmute_copy(&ok_array))
        }
    }
}

pub trait ResultVecExt {
    type T;
    type E;
    
    fn flatten_ok(self) -> Result<Vec<Self::T>, Self::E>;
}

impl <T, E> ResultVecExt for Vec<Result<T, E>> {
    type T = T;
    type E = E;
    
    fn flatten_ok(self) -> Result<Vec<Self::T>, Self::E> {
        let mut ok_vec = Vec::with_capacity(self.len());
        
        for result in self {
            match result {
                Ok(value) => ok_vec.push(value),
                Err(error) => return Err(error),
            }
        }
        
        Ok(ok_vec)   
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_ok_ok() {
        let result: [Result<i32, ()>; 5] = [Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)];

        let result = result.flatten_ok();

        assert_eq!(result, Ok([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_flatten_ok_err() {
        let result: [Result<i32, i32>; 5] = [Ok(1), Err(1), Ok(3), Err(2), Ok(5)];

        let result = result.flatten_ok();

        assert_eq!(result, Err(1));
    }
}
