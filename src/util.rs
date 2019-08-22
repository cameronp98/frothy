pub struct Cursor<'a, T> {
    items: &'a [T],
    pos: usize,
}

impl<'a, T> Cursor<'a, T> {
    fn new(items: &'a [T]) -> Cursor<'a, T> {
        Cursor {
            items,
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&T> {
        if self.pos < self.items.len() {
            Some(&self.items[self.pos])
        } else {
            None
        }
    }

    fn back(&mut self) -> Option<&T> {
        if self.pos > 0 {
            self.pos -= 1;
            Some(&self.items[self.pos])
        } else {
            None
        }
    }
}

// pop `n` args from the stack, then call `f` with those args and push the result
//
// instead of popping each arg then reversing the order, this function just gets a slice
// reference to the top of the stack, calls the function, then removes the args after
pub fn pop_n<T, F: Fn(&[T]) -> T>(s: &mut Vec<T>, n: usize, f: F) -> Result<(), String> {
    let len = s.len();

    // make sure there are enough arguments on the stack
    if len < n {
        return Err(format!("Expected at least {} arguments, stack size = {}", n, len));
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
