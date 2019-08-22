//! Utility functions for working with stacks and other postfix operations

use crate::error::{Error, Result};

// pop `n` args from the stack, then call `f` with those args and push the result
//
// instead of popping each arg then reversing the order, this function just gets a slice
// reference to the top of the stack, calls the function, then removes the args after
pub fn call<T, F: Fn(&[T]) -> T>(s: &mut Vec<T>, n: usize, f: F) -> Result<()> {
    let len = s.len();

    // make sure there are enough arguments on the stack
    if len < n {
        return Err(Error::NotEnoughArguments(n, len));
    }

    // create the range index for the `n` args on the top of the stack
    let args_start = len - n;

    // call the function with the args
    let result = f(&s[args_start..]);

    // 'pop' the args from the stack
    s.drain(args_start..);

    // push the result
    s.push(result);

    Ok(())
}
