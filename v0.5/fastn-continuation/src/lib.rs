pub enum Result<C: Continuation + ?Sized> {
    Done(C::Output),
    Stuck(Box<C>, C::NeededInput),
}

pub trait Continuation {
    type Output;
    type NeededInput;
    type NeededOutput;
    fn continue_after(self, n: Self::NeededOutput) -> Result<Self>;
}

impl<C: Continuation> Result<C> {
    pub fn consume<F>(mut self, f: F) -> C::Output
    where
        F: Fn(C::NeededInput) -> C::NeededOutput,
    {
        loop {
            match self {
                Result::Stuck(ic, input) => {
                    self = ic.continue_after(f(input));
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub fn consume_with<F>(mut self, f: F) -> C::Output
    where
        F: Fn(&mut C, C::NeededInput) -> C::NeededOutput,
    {
        loop {
            match self {
                Result::Stuck(mut ic, input) => {
                    let o = f(&mut ic, input);
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_async<Fut>(mut self, f: impl Fn(C::NeededInput) -> Fut) -> C::Output
    where
        Fut: std::future::Future<Output = C::NeededOutput>,
    {
        loop {
            match self {
                Result::Stuck(ic, input) => {
                    self = ic.continue_after(f(input).await);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }

    pub async fn consume_with_async<Fut>(
        mut self,
        f: impl Fn(&mut C, C::NeededInput) -> Fut,
    ) -> C::Output
    where
        Fut: std::future::Future<Output = C::NeededOutput>,
    {
        loop {
            match self {
                Result::Stuck(mut ic, input) => {
                    let o = f(&mut ic, input).await;
                    self = ic.continue_after(o);
                }
                Result::Done(c) => {
                    return c;
                }
            }
        }
    }
}
