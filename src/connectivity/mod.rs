pub mod axum_example;
pub mod warp_example;
pub mod reqwest_example;

#[async_trait::async_trait]
pub trait Trait {
    async fn f(&self);
}

pub fn make() -> Box<dyn Trait> {
    unimplemented!()
}
