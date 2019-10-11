pub trait Element {
    type Input: Sized + Send;
    type Output: Sized + Send;

    fn process(&mut self, packet: Self::Input) -> Option<Self::Output>;
}

pub trait AsyncElement {
    type Input: Sized;
    type Output: Sized;

    fn process(&mut self, packet: Self::Input) -> Option<Self::Output>;
}
